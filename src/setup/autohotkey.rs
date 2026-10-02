//! Platform-independent AutoHotkey rendering and recoverable setup lifecycle.
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) type Result<T> = std::result::Result<T, Box<dyn Error>>;
pub(super) const MARKER: &str = "; Managed by dedent-paste --install (AutoHotkey).";
#[cfg(target_os = "windows")]
pub(super) const SHORTCUT_DESCRIPTION: &str = "dedent-paste managed Win+V shortcut";

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Version {
    V1,
    V2,
}

impl Version {
    pub(super) fn from_parts(major: u64, minor: u64) -> Option<Self> {
        match (major, minor) {
            (1, 1) => Some(Self::V1),
            (2, _) => Some(Self::V2),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct Interpreter {
    pub path: PathBuf,
    pub version: Version,
}

pub(super) fn select_interpreter(candidates: Vec<Interpreter>) -> Result<Interpreter> {
    // Keep discovery order within each version (per-user before system paths).
    candidates.iter().find(|i| i.version == Version::V2).cloned()
        .or_else(|| candidates.into_iter().next())
        .ok_or_else(|| "AutoHotkey v2 or v1.1 was not found. Install AutoHotkey from https://www.autohotkey.com/ (or put its interpreter on PATH), then run 'dedent-paste --install' again.".into())
}

pub(super) fn render_script(binary: &Path, version: Version) -> Result<Vec<u8>> {
    let binary = binary
        .to_str()
        .ok_or("executable path is not valid Unicode")?;
    if binary.contains(['\r', '\n', '\0', '"']) {
        return Err("executable path contains unsupported characters".into());
    }
    let template = match version {
        Version::V1 => include_str!("../../examples/windows/dedent-paste-win-v-v1.ahk"),
        Version::V2 => include_str!("../../examples/windows/dedent-paste-win-v-v2.ahk"),
    };
    let mut output = format!(
        "\u{feff}{MARKER}\n; Re-running --install replaces this script after backing it up.\n"
    );
    for line in template.lines() {
        if line.starts_with("dedentPaste := ") {
            output.push_str(&format!(
                "dedentPaste := \"{}\"\n",
                binary.replace('`', "``")
            ));
        } else {
            output.push_str(line);
            output.push('\n');
        }
    }
    Ok(output.into_bytes())
}

pub(super) fn is_script_window(class: &str, title: &str, script: &str) -> bool {
    class == "AutoHotkey"
        && title
            .to_lowercase()
            .starts_with(&format!("{script} - AutoHotkey v").to_lowercase())
}

pub(super) struct Paths {
    pub script: PathBuf,
    pub shortcut: PathBuf,
}

/// Desktop operations are injected so failure recovery can be tested on any OS.
pub(super) trait Desktop {
    fn validate_shortcut(&mut self, paths: &Paths) -> Result<()>;
    fn create_shortcut(&mut self, paths: &Paths, interpreter: &Path) -> Result<()>;
    fn running_interpreter(&mut self, script: &Path) -> Result<Option<PathBuf>>;
    fn stop(&mut self, script: &Path) -> Result<()>;
    fn start(&mut self, script: &Path, interpreter: &Path) -> Result<()>;
}

pub(super) fn install(
    paths: &Paths,
    binary: &Path,
    interpreter: &Interpreter,
    desktop: &mut impl Desktop,
) -> Result<()> {
    let script = render_script(binary, interpreter.version)?;
    let previous = Snapshot::read(paths, desktop)?;
    let running = desktop.running_interpreter(&paths.script)?;
    for path in [&paths.script, &paths.shortcut] {
        fs::create_dir_all(path.parent().ok_or("setup path has no parent")?)?;
    }
    if previous.script.as_deref().is_some_and(|old| old != script) {
        backup(&paths.script)?;
    }
    desktop.stop(&paths.script)?;
    let result = (|| {
        atomic_write(&paths.script, &script)?;
        desktop.create_shortcut(paths, &interpreter.path)?;
        desktop.start(&paths.script, &interpreter.path)
    })();
    if let Err(error) = result {
        return Err(recover(error, paths, previous, running, desktop));
    }
    Ok(())
}

pub(super) fn uninstall(paths: &Paths, desktop: &mut impl Desktop) -> Result<()> {
    let previous = Snapshot::read(paths, desktop)?;
    let running = desktop.running_interpreter(&paths.script)?;
    if previous.script.is_some() {
        backup(&paths.script)?;
    }
    desktop.stop(&paths.script)?;
    let result = (|| {
        remove_if_present(&paths.shortcut)?;
        remove_if_present(&paths.script)
    })();
    if let Err(error) = result {
        return Err(recover(error, paths, previous, running, desktop));
    }
    Ok(())
}

fn read_optional(path: &Path) -> Result<Option<Vec<u8>>> {
    // Do not follow a symlink/reparse point into someone else's configuration.
    match fs::symlink_metadata(path) {
        Ok(metadata) if !metadata.file_type().is_file() => {
            return Err(format!("refusing to modify non-regular file: {}", path.display()).into());
        }
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    }
    Ok(Some(fs::read(path).map_err(|e| {
        format!("failed to read {}: {e}", path.display())
    })?))
}

struct Snapshot {
    script: Option<Vec<u8>>,
    shortcut: Option<Vec<u8>>,
}

impl Snapshot {
    fn read(paths: &Paths, desktop: &mut impl Desktop) -> Result<Self> {
        let script = read_optional(&paths.script)?;
        if let Some(bytes) = &script {
            let text = std::str::from_utf8(bytes)
                .unwrap_or("")
                .trim_start_matches('\u{feff}');
            if text.lines().next() != Some(MARKER) {
                return Err(format!(
                    "{} is not a dedent-paste managed script; move it aside before running setup",
                    paths.script.display()
                )
                .into());
            }
        }
        let shortcut = read_optional(&paths.shortcut)?;
        desktop.validate_shortcut(paths)?;
        Ok(Self { script, shortcut })
    }
}

fn remove_if_present(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("failed to remove {}: {e}", path.display()).into()),
    }
}

fn recover(
    error: Box<dyn Error>,
    paths: &Paths,
    previous: Snapshot,
    running: Option<PathBuf>,
    desktop: &mut impl Desktop,
) -> Box<dyn Error> {
    let mut failures = Vec::new();
    if let Err(e) = desktop.stop(&paths.script) {
        failures.push(format!("stop managed script: {e}"));
    }
    for (path, bytes) in [
        (&paths.script, previous.script),
        (&paths.shortcut, previous.shortcut),
    ] {
        let result = match bytes {
            Some(bytes) => atomic_write(path, &bytes),
            None => remove_if_present(path),
        };
        if let Err(e) = result {
            failures.push(format!("restore {}: {e}", path.display()));
        }
    }
    if failures.is_empty() {
        if let Some(interpreter) = running {
            if let Err(e) = desktop.start(&paths.script, &interpreter) {
                failures.push(format!("restart previous script: {e}"));
            }
        }
    }
    if failures.is_empty() {
        format!("{error}. Previous setup was restored.").into()
    } else {
        format!(
            "{error}. Recovery incomplete: {}. Inspect the managed files before retrying.",
            failures.join("; ")
        )
        .into()
    }
}

fn unique_file(path: &Path, kind: &str) -> Result<(PathBuf, fs::File)> {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    for counter in 0..1000 {
        let mut name = path.as_os_str().to_os_string();
        name.push(format!(".{kind}-{stamp}-{counter}"));
        let candidate = PathBuf::from(name);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => return Ok((candidate, file)),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(format!("failed to create {}: {e}", candidate.display()).into()),
        }
    }
    Err("could not allocate a unique setup file".into())
}

fn backup(path: &Path) -> Result<()> {
    let bytes = fs::read(path)?;
    let (backup, mut file) = unique_file(path, "bak")?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    println!("Backed up managed script: {}", backup.display());
    Ok(())
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let (temp, mut file) = unique_file(path, "tmp")?;
    let result = (|| -> std::io::Result<()> {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp, path)
    })();
    if let Err(error) = result {
        let cleanup = remove_if_present(&temp);
        return Err(format!(
            "failed to write {}: {error}{}",
            path.display(),
            cleanup
                .err()
                .map(|e| format!("; temporary file cleanup failed: {e}"))
                .unwrap_or_default()
        )
        .into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let dir = std::env::temp_dir().join(format!(
                "dedent-paste-setup-{}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&dir).unwrap();
            Self(dir)
        }
        fn paths(&self) -> Paths {
            Paths {
                script: self.0.join("script.ahk"),
                shortcut: self.0.join("startup/link.lnk"),
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[derive(Default)]
    struct FakeDesktop {
        running: Option<PathBuf>,
        fail_start: bool,
        fail_shortcut: bool,
        fail_stop: bool,
        starts: usize,
        fail_recovery: bool,
    }
    impl Desktop for FakeDesktop {
        fn validate_shortcut(&mut self, paths: &Paths) -> Result<()> {
            if paths.shortcut.exists() && fs::read(&paths.shortcut)? != b"managed shortcut" {
                return Err("unrelated shortcut".into());
            }
            Ok(())
        }
        fn create_shortcut(&mut self, paths: &Paths, _: &Path) -> Result<()> {
            fs::write(&paths.shortcut, b"managed shortcut")?;
            if self.fail_shortcut {
                return Err("shortcut failed".into());
            }
            Ok(())
        }
        fn running_interpreter(&mut self, _: &Path) -> Result<Option<PathBuf>> {
            Ok(self.running.clone())
        }
        fn stop(&mut self, _: &Path) -> Result<()> {
            if self.fail_stop {
                return Err("stop failed".into());
            }
            self.running = None;
            Ok(())
        }
        fn start(&mut self, _: &Path, interpreter: &Path) -> Result<()> {
            self.starts += 1;
            self.running = Some(interpreter.into());
            if std::mem::take(&mut self.fail_start) {
                self.fail_stop = self.fail_recovery;
                return Err("launch failed".into());
            }
            Ok(())
        }
    }
    fn interpreter(version: Version) -> Interpreter {
        Interpreter {
            path: "AutoHotkey.exe".into(),
            version,
        }
    }

    #[test]
    fn selects_v2_and_rejects_unsupported_versions() {
        assert_eq!(
            select_interpreter(vec![interpreter(Version::V1), interpreter(Version::V2)])
                .unwrap()
                .version,
            Version::V2
        );
        assert_eq!(
            select_interpreter(vec![interpreter(Version::V1)])
                .unwrap()
                .version,
            Version::V1
        );
        assert!(
            select_interpreter(vec![])
                .unwrap_err()
                .to_string()
                .contains("Install AutoHotkey")
        );
        assert_eq!(Version::from_parts(1, 0), None);
        assert_eq!(Version::from_parts(1, 1), Some(Version::V1));
        assert_eq!(Version::from_parts(2, 0), Some(Version::V2));
        assert_eq!(Version::from_parts(3, 0), None);
    }

    #[test]
    fn window_match_requires_full_script_path_and_class() {
        let path = r"C:\Users\使用者\dedent-paste-win-v.ahk";
        assert!(is_script_window(
            "AutoHotkey",
            &format!("{path} - AutoHotkey v2.0.19"),
            path
        ));
        assert!(is_script_window(
            "AutoHotkey",
            &format!("{path} - AutoHotkey v1.1.37.02").to_uppercase(),
            path
        ));
        assert!(!is_script_window(
            "OtherApp",
            &format!("{path} - AutoHotkey v2.0.19"),
            path
        ));
        assert!(!is_script_window(
            "AutoHotkey",
            &format!("{path}.other.ahk - AutoHotkey v2.0.19"),
            path
        ));
        assert!(!is_script_window(
            "AutoHotkey",
            "dedent-paste-win-v.ahk - AutoHotkey v2.0.19",
            path
        ));
    }

    #[test]
    fn repeated_backups_are_distinct_and_unchanged_reinstall_needs_none() {
        let fixture = Fixture::new();
        let paths = fixture.paths();
        let mut desktop = FakeDesktop::default();
        install(
            &paths,
            Path::new("bin.exe"),
            &interpreter(Version::V2),
            &mut desktop,
        )
        .unwrap();
        install(
            &paths,
            Path::new("bin.exe"),
            &interpreter(Version::V2),
            &mut desktop,
        )
        .unwrap();
        let backup_count = || {
            fs::read_dir(&fixture.0)
                .unwrap()
                .filter(|e| {
                    e.as_ref()
                        .unwrap()
                        .file_name()
                        .to_string_lossy()
                        .contains(".bak-")
                })
                .count()
        };
        assert_eq!(backup_count(), 0);
        backup(&paths.script).unwrap();
        backup(&paths.script).unwrap();
        assert_eq!(backup_count(), 2);
    }

    #[test]
    fn incomplete_recovery_is_explicit_and_files_are_restored() {
        let fixture = Fixture::new();
        let paths = fixture.paths();
        let mut desktop = FakeDesktop {
            fail_start: true,
            fail_recovery: true,
            ..Default::default()
        };
        let error = install(
            &paths,
            Path::new("bin.exe"),
            &interpreter(Version::V2),
            &mut desktop,
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("launch failed"));
        assert!(error.contains("Recovery incomplete"));
        assert!(error.contains("stop failed"));
        assert!(!paths.script.exists());
        assert!(!paths.shortcut.exists());
    }

    #[test]
    fn uninstall_removes_orphaned_owned_shortcut_without_interpreter() {
        let fixture = Fixture::new();
        let paths = fixture.paths();
        fs::create_dir_all(paths.shortcut.parent().unwrap()).unwrap();
        fs::write(&paths.shortcut, b"managed shortcut").unwrap();
        let mut desktop = FakeDesktop::default();
        uninstall(&paths, &mut desktop).unwrap();
        assert!(!paths.shortcut.exists());
        assert_eq!(desktop.starts, 0);
    }

    #[test]
    fn renders_versions_with_unicode_and_literal_backticks() {
        let binary = Path::new(r"C:\使用者\O'Brien `n %path%\dedent-paste.exe");
        for version in [Version::V1, Version::V2] {
            let bytes = render_script(binary, version).unwrap();
            assert!(bytes.starts_with(b"\xef\xbb\xbf"));
            let text = String::from_utf8(bytes).unwrap();
            assert!(text.contains(MARKER));
            assert!(
                text.contains(r#"dedentPaste := "C:\使用者\O'Brien ``n %path%\dedent-paste.exe""#)
            );
            assert!(!text.contains("A_Home"));
            assert!(text.contains("#SingleInstance Force"));
            assert!(text.contains("RunWait"));
            assert!(text.contains("KeyWait"));
        }
        assert!(render_script(Path::new("bad\npath"), Version::V2).is_err());
        assert!(render_script(Path::new("bad\"path"), Version::V1).is_err());
    }

    #[test]
    fn install_reinstall_and_uninstall_preserve_edits_in_backups() {
        let fixture = Fixture::new();
        let paths = fixture.paths();
        let mut desktop = FakeDesktop::default();
        install(
            &paths,
            Path::new("old.exe"),
            &interpreter(Version::V1),
            &mut desktop,
        )
        .unwrap();
        let mut edited = fs::read(&paths.script).unwrap();
        edited.extend_from_slice(b"\n; my edits\n");
        fs::write(&paths.script, &edited).unwrap();
        install(
            &paths,
            Path::new("new.exe"),
            &interpreter(Version::V2),
            &mut desktop,
        )
        .unwrap();
        assert!(
            fs::read_to_string(&paths.script)
                .unwrap()
                .contains("new.exe")
        );
        assert!(
            fs::read_dir(&fixture.0)
                .unwrap()
                .any(|entry| fs::read(entry.unwrap().path()).ok().as_ref() == Some(&edited))
        );
        uninstall(&paths, &mut desktop).unwrap();
        uninstall(&paths, &mut desktop).unwrap();
        assert!(!paths.script.exists());
        assert!(!paths.shortcut.exists());
        assert!(desktop.running.is_none());
    }

    #[test]
    fn failed_launch_restores_previous_files_and_running_interpreter() {
        let fixture = Fixture::new();
        let paths = fixture.paths();
        let mut desktop = FakeDesktop::default();
        install(
            &paths,
            Path::new("old.exe"),
            &interpreter(Version::V1),
            &mut desktop,
        )
        .unwrap();
        let before = fs::read(&paths.script).unwrap();
        desktop.fail_start = true;
        assert!(
            install(
                &paths,
                Path::new("new.exe"),
                &interpreter(Version::V2),
                &mut desktop
            )
            .is_err()
        );
        assert_eq!(fs::read(&paths.script).unwrap(), before);
        assert_eq!(fs::read(&paths.shortcut).unwrap(), b"managed shortcut");
        assert_eq!(desktop.running, Some(PathBuf::from("AutoHotkey.exe")));
        assert_eq!(desktop.starts, 3);
    }

    #[test]
    fn failed_first_install_removes_partial_files() {
        for fail_shortcut in [false, true] {
            let fixture = Fixture::new();
            let paths = fixture.paths();
            let mut desktop = FakeDesktop {
                fail_start: !fail_shortcut,
                fail_shortcut,
                ..Default::default()
            };
            assert!(
                install(
                    &paths,
                    Path::new("bin.exe"),
                    &interpreter(Version::V2),
                    &mut desktop
                )
                .is_err()
            );
            assert!(!paths.script.exists());
            assert!(!paths.shortcut.exists());
            assert!(desktop.running.is_none());
        }
    }

    #[test]
    fn unrelated_files_and_stop_failures_are_not_overwritten() {
        let fixture = Fixture::new();
        let paths = fixture.paths();
        let mut desktop = FakeDesktop::default();
        fs::write(&paths.script, b"user script").unwrap();
        assert!(
            install(
                &paths,
                Path::new("bin.exe"),
                &interpreter(Version::V2),
                &mut desktop
            )
            .is_err()
        );
        assert!(uninstall(&paths, &mut desktop).is_err());
        assert_eq!(fs::read(&paths.script).unwrap(), b"user script");
        fs::remove_file(&paths.script).unwrap();
        install(
            &paths,
            Path::new("bin.exe"),
            &interpreter(Version::V2),
            &mut desktop,
        )
        .unwrap();
        let before = fs::read(&paths.script).unwrap();
        desktop.fail_stop = true;
        assert!(uninstall(&paths, &mut desktop).is_err());
        assert_eq!(fs::read(&paths.script).unwrap(), before);
        desktop.fail_stop = false;
        fs::write(&paths.shortcut, b"user shortcut").unwrap();
        assert!(
            install(
                &paths,
                Path::new("new.exe"),
                &interpreter(Version::V2),
                &mut desktop
            )
            .is_err()
        );
        assert!(uninstall(&paths, &mut desktop).is_err());
        assert_eq!(fs::read(&paths.shortcut).unwrap(), b"user shortcut");
    }
}
