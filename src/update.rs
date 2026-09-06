use std::cmp::Ordering;
use std::error::Error;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallMethod {
    Homebrew,
    Npm,
    Cargo,
    LocalBuild,
    Standalone,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UpdateOptions {
    pub check: bool,
    pub force: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemVer {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub prerelease: Option<String>,
}

impl SemVer {
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim().strip_prefix('v').unwrap_or(s.trim());
        let (ver, pre) = match s.split_once('-') {
            Some((v, p)) => (v, Some(p.to_string())),
            None => (s, None),
        };
        let mut parts = ver.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;
        if parts.next().is_some() {
            return None;
        }
        Some(Self {
            major,
            minor,
            patch,
            prerelease: pre,
        })
    }
}

impl PartialOrd for SemVer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemVer {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch)) {
            Ordering::Equal => match (&self.prerelease, &other.prerelease) {
                (None, None) => Ordering::Equal,
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (Some(a), Some(b)) => a.cmp(b),
            },
            other => other,
        }
    }
}

pub fn is_newer_version(current: &str, latest: &str) -> bool {
    let (Some(current_ver), Some(latest_ver)) = (SemVer::parse(current), SemVer::parse(latest))
    else {
        return false;
    };
    latest_ver > current_ver
}

pub fn detect_install_method(exe_path: &Path) -> InstallMethod {
    if let Ok(canonical) = std::fs::canonicalize(exe_path) {
        let method = detect_install_method_str(&canonical.to_string_lossy());
        if method != InstallMethod::Standalone {
            return method;
        }
    }
    detect_install_method_str(&exe_path.to_string_lossy())
}

pub fn detect_install_method_str(path: &str) -> InstallMethod {
    let normalized = path.replace('\\', "/").to_lowercase();
    if normalized.contains("/target/debug/") || normalized.contains("/target/release/") {
        return InstallMethod::LocalBuild;
    }
    if normalized.contains("/cellar/")
        || normalized.contains("/opt/homebrew/")
        || normalized.contains("/.linuxbrew/")
    {
        return InstallMethod::Homebrew;
    }
    if normalized.contains("/node_modules/")
        || normalized.contains("/.nvm/")
        || normalized.contains("/npm/")
        || normalized.contains("/npm-global/")
    {
        return InstallMethod::Npm;
    }
    if normalized.contains("/.cargo/bin/") {
        return InstallMethod::Cargo;
    }
    InstallMethod::Standalone
}

pub fn extract_latest_version(json_str: &str) -> Option<String> {
    let value: serde_json::Value = serde_json::from_str(json_str).ok()?;

    // 1. Try dist-manifest.json announcement_tag
    if let Some(tag) = value.get("announcement_tag").and_then(|v| v.as_str()) {
        let clean = tag.strip_prefix('v').unwrap_or(tag);
        if SemVer::parse(clean).is_some() {
            return Some(clean.to_string());
        }
    }

    // 2. Try dist-manifest.json releases[0].app_version
    if let Some(ver) = value
        .get("releases")
        .and_then(|r| r.as_array())
        .and_then(|a| a.first())
        .and_then(|rel| rel.get("app_version"))
        .and_then(|v| v.as_str())
    {
        let clean = ver.strip_prefix('v').unwrap_or(ver);
        if SemVer::parse(clean).is_some() {
            return Some(clean.to_string());
        }
    }

    // 3. Try GitHub API release tag_name
    if let Some(tag) = value.get("tag_name").and_then(|v| v.as_str()) {
        let clean = tag.strip_prefix('v').unwrap_or(tag);
        if SemVer::parse(clean).is_some() {
            return Some(clean.to_string());
        }
    }

    // 4. Try GitHub API release name
    if let Some(name) = value.get("name").and_then(|v| v.as_str()) {
        let clean = name.strip_prefix('v').unwrap_or(name);
        if SemVer::parse(clean).is_some() {
            return Some(clean.to_string());
        }
    }

    None
}

const RESPONSE_BODY_LIMIT: u64 = 10 * 1024 * 1024;

const DEFAULT_DIST_MANIFEST_URL: &str =
    "https://github.com/doggy8088/dedent-paste/releases/latest/download/dist-manifest.json";
const DEFAULT_GITHUB_API_URL: &str =
    "https://api.github.com/repos/doggy8088/dedent-paste/releases/latest";

fn build_ureq_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(15)))
        .http_status_as_error(false)
        .build()
        .into()
}

fn http_get(url: &str) -> Result<(u16, String), Box<dyn Error>> {
    let agent = build_ureq_agent();
    let mut req = agent.get(url).header(
        "User-Agent",
        format!("dedent-paste/{}", env!("CARGO_PKG_VERSION")),
    );
    if let Ok(token) =
        std::env::var("DEDENT_PASTE_GITHUB_TOKEN").or_else(|_| std::env::var("GITHUB_TOKEN"))
    {
        let token = token.trim();
        if !token.is_empty() {
            req = req.header("Authorization", format!("Bearer {token}"));
        }
    }
    let mut resp = req.call()?;
    let status = resp.status().as_u16();
    let body = resp
        .body_mut()
        .with_config()
        .limit(RESPONSE_BODY_LIMIT)
        .read_to_string()?;
    Ok((status, body))
}

pub fn fetch_latest_version() -> Result<String, Box<dyn Error>> {
    // First attempt: dist-manifest.json (release asset, no GitHub API rate limits)
    let manifest_url = std::env::var("DEDENT_PASTE_DIST_MANIFEST_URL")
        .unwrap_or_else(|_| DEFAULT_DIST_MANIFEST_URL.to_string());
    if let Ok((status, body)) = http_get(&manifest_url) {
        if (200..300).contains(&status) {
            if let Some(ver) = extract_latest_version(&body) {
                return Ok(ver);
            }
        }
    }

    // Fallback: GitHub Releases API
    let api_url = std::env::var("DEDENT_PASTE_GITHUB_API_URL")
        .unwrap_or_else(|_| DEFAULT_GITHUB_API_URL.to_string());
    let (status, body) =
        http_get(&api_url).map_err(|e| format!("failed to check for updates: {e}"))?;
    if !(200..300).contains(&status) {
        return Err(format!("failed to check for updates from {api_url} (HTTP {status})").into());
    }
    extract_latest_version(&body)
        .ok_or_else(|| "could not determine latest version from release metadata".into())
}

pub fn run_update(options: &UpdateOptions) -> Result<(), Box<dyn Error>> {
    let current_exe = std::env::current_exe()?;
    let install_method = detect_install_method(&current_exe);
    let current_version = env!("CARGO_PKG_VERSION");

    println!("Checking for updates...");
    let latest_version = fetch_latest_version()?;
    let update_available = is_newer_version(current_version, &latest_version);

    if options.check {
        if update_available {
            println!(
                "A new version of dedent-paste is available: v{current_version} -> v{latest_version}"
            );
            match install_method {
                InstallMethod::Homebrew => println!("To update, run: brew upgrade dedent-paste"),
                InstallMethod::Npm => println!("To update, run: npm install -g dedent-paste"),
                InstallMethod::Cargo => {
                    println!("To update, run: cargo install --locked dedent-paste")
                }
                InstallMethod::LocalBuild => println!(
                    "dedent-paste is running from a local build ({})",
                    current_exe.display()
                ),
                InstallMethod::Standalone => println!("To update, run: dedent-paste update"),
            }
        } else {
            println!("dedent-paste is up to date (v{current_version}).");
        }
        return Ok(());
    }

    if !update_available && !options.force {
        println!("dedent-paste is already up to date (v{current_version}).");
        return Ok(());
    }

    match install_method {
        InstallMethod::Homebrew => {
            return Err(
                "dedent-paste was installed via Homebrew. Please run: brew upgrade dedent-paste"
                    .into(),
            );
        }
        InstallMethod::Npm => {
            return Err(
                "dedent-paste was installed via npm. Please run: npm install -g dedent-paste"
                    .into(),
            );
        }
        InstallMethod::Cargo => {
            return Err(
                "dedent-paste was installed via Cargo. Please run: cargo install --locked dedent-paste"
                    .into(),
            );
        }
        InstallMethod::LocalBuild => {
            return Err(format!(
                "dedent-paste is running from a local build ({}). Skipping self-update.",
                current_exe.display()
            )
            .into());
        }
        InstallMethod::Standalone => {}
    }

    println!("Updating dedent-paste from v{current_version} to v{latest_version}...");
    perform_standalone_update(&current_exe, &latest_version)?;
    println!("dedent-paste updated to v{latest_version} successfully!");
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn perform_standalone_update(
    current_exe: &Path,
    _latest_version: &str,
) -> Result<(), Box<dyn Error>> {
    use std::io::Write as _;
    use std::process::{Command, Stdio};

    let install_dir = current_exe
        .parent()
        .ok_or("failed to determine executable directory")?;

    let base_url = std::env::var("DEDENT_PASTE_INSTALLER_GITHUB_BASE_URL")
        .unwrap_or_else(|_| "https://github.com".to_string());
    let script_url = format!(
        "{base_url}/doggy8088/dedent-paste/releases/latest/download/dedent-paste-installer.sh"
    );

    let (status, script) =
        http_get(&script_url).map_err(|e| format!("failed to download installer script: {e}"))?;
    if !(200..300).contains(&status) {
        return Err(format!(
            "failed to download installer script from {script_url} (HTTP {status})"
        )
        .into());
    }

    let mut child = Command::new("sh")
        .arg("-s")
        .arg("--")
        .arg("--quiet")
        .env("DEDENT_PASTE_INSTALL_DIR", install_dir)
        .env("INSTALLER_NO_MODIFY_PATH", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("failed to spawn sh: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(script.as_bytes())?;
    }

    let exit_status = child.wait()?;
    if !exit_status.success() {
        return Err(format!("installer failed with exit status {exit_status}").into());
    }

    #[cfg(target_os = "macos")]
    {
        // Re-run --install so Karabiner rules stay in sync with the newly installed binary.
        let _ = Command::new(current_exe).arg("--install").status();
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn perform_standalone_update(
    current_exe: &Path,
    _latest_version: &str,
) -> Result<(), Box<dyn Error>> {
    use std::process::Command;

    let install_dir = current_exe
        .parent()
        .ok_or("failed to determine executable directory")?;

    let base_url = std::env::var("DEDENT_PASTE_INSTALLER_GITHUB_BASE_URL")
        .unwrap_or_else(|_| "https://github.com".to_string());
    let script_url = format!(
        "{base_url}/doggy8088/dedent-paste/releases/latest/download/dedent-paste-installer.ps1"
    );

    let (status, script) =
        http_get(&script_url).map_err(|e| format!("failed to download installer script: {e}"))?;
    if !(200..300).contains(&status) {
        return Err(format!(
            "failed to download installer script from {script_url} (HTTP {status})"
        )
        .into());
    }

    let temp_script =
        std::env::temp_dir().join(format!("dedent-paste-installer-{}.ps1", std::process::id()));
    std::fs::write(&temp_script, &script)
        .map_err(|e| format!("failed to write temporary installer script: {e}"))?;

    let old_exe = current_exe.with_extension("exe.old");
    let _ = std::fs::remove_file(&old_exe);
    std::fs::rename(current_exe, &old_exe)
        .map_err(|e| format!("failed to prepare executable for replacement: {e}"))?;

    let ps_result = Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            temp_script.to_str().unwrap(),
            "-NoModifyPath",
        ])
        .env("DEDENT_PASTE_INSTALL_DIR", install_dir)
        .status();

    let _ = std::fs::remove_file(&temp_script);

    match ps_result {
        Ok(status) if status.success() => {
            let _ = std::fs::remove_file(&old_exe);
            Ok(())
        }
        Ok(status) => {
            let _ = std::fs::rename(&old_exe, current_exe);
            Err(format!("PowerShell installer failed with exit code: {status}").into())
        }
        Err(err) => {
            let _ = std::fs::rename(&old_exe, current_exe);
            Err(format!("failed to execute powershell: {err}").into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semver_parses_valid_strings() {
        assert_eq!(
            SemVer::parse("0.5.0"),
            Some(SemVer {
                major: 0,
                minor: 5,
                patch: 0,
                prerelease: None,
            })
        );
        assert_eq!(
            SemVer::parse("v1.2.3"),
            Some(SemVer {
                major: 1,
                minor: 2,
                patch: 3,
                prerelease: None,
            })
        );
        assert_eq!(
            SemVer::parse(" 0.6.0-rc.1 "),
            Some(SemVer {
                major: 0,
                minor: 6,
                patch: 0,
                prerelease: Some("rc.1".to_string()),
            })
        );
    }

    #[test]
    fn semver_rejects_invalid_strings() {
        assert_eq!(SemVer::parse(""), None);
        assert_eq!(SemVer::parse("abc"), None);
        assert_eq!(SemVer::parse("1.2"), None);
        assert_eq!(SemVer::parse("1.2.3.4"), None);
    }

    #[test]
    fn version_comparison_logic() {
        assert!(is_newer_version("0.5.0", "0.5.1"));
        assert!(is_newer_version("0.5.0", "v0.5.1"));
        assert!(is_newer_version("0.5.0", "0.6.0"));
        assert!(is_newer_version("0.5.0", "1.0.0"));
        assert!(!is_newer_version("0.5.0", "0.5.0"));
        assert!(!is_newer_version("0.5.1", "0.5.0"));
        assert!(!is_newer_version("1.0.0", "0.9.9"));
        assert!(is_newer_version("0.5.0-alpha", "0.5.0"));
        assert!(!is_newer_version("0.5.0", "0.5.0-alpha"));
    }

    #[test]
    fn install_method_detection() {
        assert_eq!(
            detect_install_method_str("/opt/homebrew/Cellar/dedent-paste/0.5.0/bin/dedent-paste"),
            InstallMethod::Homebrew
        );
        assert_eq!(
            detect_install_method_str("/opt/homebrew/bin/dedent-paste"),
            InstallMethod::Homebrew
        );
        assert_eq!(
            detect_install_method_str("/usr/local/Cellar/dedent-paste/0.5.0/bin/dedent-paste"),
            InstallMethod::Homebrew
        );
        assert_eq!(
            detect_install_method_str("/home/linuxbrew/.linuxbrew/bin/dedent-paste"),
            InstallMethod::Homebrew
        );
        assert_eq!(
            detect_install_method_str(
                "C:\\Users\\user\\AppData\\Roaming\\npm\\node_modules\\dedent-paste\\bin\\dedent-paste.exe"
            ),
            InstallMethod::Npm
        );
        assert_eq!(
            detect_install_method_str("/home/user/.nvm/versions/node/v20.0.0/bin/dedent-paste"),
            InstallMethod::Npm
        );
        assert_eq!(
            detect_install_method_str("/usr/local/lib/node_modules/dedent-paste/dedent-paste"),
            InstallMethod::Npm
        );
        assert_eq!(
            detect_install_method_str("/Users/will/.cargo/bin/dedent-paste"),
            InstallMethod::Cargo
        );
        assert_eq!(
            detect_install_method_str("C:\\Users\\will\\.cargo\\bin\\dedent-paste.exe"),
            InstallMethod::Cargo
        );
        assert_eq!(
            detect_install_method_str(
                "/Users/will/projects/dedent-paste/target/release/dedent-paste"
            ),
            InstallMethod::LocalBuild
        );
        assert_eq!(
            detect_install_method_str(
                "/Users/will/projects/dedent-paste/target/debug/dedent-paste"
            ),
            InstallMethod::LocalBuild
        );
        assert_eq!(
            detect_install_method_str("/Users/will/.local/bin/dedent-paste"),
            InstallMethod::Standalone
        );
        assert_eq!(
            detect_install_method_str("C:\\Users\\will\\.local\\bin\\dedent-paste.exe"),
            InstallMethod::Standalone
        );
    }

    #[test]
    fn extracts_version_from_manifest_json() {
        let manifest = r#"{
            "dist_version": "0.32.0",
            "announcement_tag": "v0.5.0",
            "releases": [
                {
                    "app_name": "dedent-paste",
                    "app_version": "0.5.0"
                }
            ]
        }"#;
        assert_eq!(extract_latest_version(manifest), Some("0.5.0".to_string()));

        let manifest_no_tag = r#"{
            "releases": [
                {
                    "app_name": "dedent-paste",
                    "app_version": "0.6.1"
                }
            ]
        }"#;
        assert_eq!(
            extract_latest_version(manifest_no_tag),
            Some("0.6.1".to_string())
        );
    }

    #[test]
    fn extracts_version_from_github_api_json() {
        let api_response = r#"{
            "tag_name": "v0.5.2",
            "name": "0.5.2"
        }"#;
        assert_eq!(
            extract_latest_version(api_response),
            Some("0.5.2".to_string())
        );

        let api_response_no_tag = r#"{
            "name": "v0.7.0"
        }"#;
        assert_eq!(
            extract_latest_version(api_response_no_tag),
            Some("0.7.0".to_string())
        );
    }

    #[test]
    fn extracts_version_handles_malformed_json() {
        assert_eq!(extract_latest_version("invalid json"), None);
        assert_eq!(extract_latest_version("{}"), None);
        assert_eq!(
            extract_latest_version(r#"{"announcement_tag": "not-a-version"}"#),
            None
        );
    }
}
