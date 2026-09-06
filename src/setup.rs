//! `--install` / `--uninstall`: file-system side of the Karabiner-Elements
//! integration. The JSON manipulation itself lives in `dedent_paste::karabiner`
//! so it can be unit tested without touching the user's configuration.

#[cfg(target_os = "macos")]
pub use macos::{install, uninstall};

#[cfg(not(target_os = "macos"))]
pub use unsupported::{install, uninstall};

#[cfg(target_os = "macos")]
mod macos {
    use std::error::Error;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    use dedent_paste::format_timestamp;
    use dedent_paste::karabiner::{
        ASSET_FILE_NAME, build_asset, build_rule, install_rule, stable_binary_path,
        to_karabiner_json, uninstall_rule,
    };
    use serde_json::Value;

    const KARABINER_APP: &str = "/Applications/Karabiner-Elements.app";
    const KARABINER_CLI: &str =
        "/Library/Application Support/org.pqrs/Karabiner-Elements/bin/karabiner_cli";

    struct Paths {
        config: PathBuf,
        asset: PathBuf,
        env_file: PathBuf,
    }

    fn paths() -> Result<Paths, Box<dyn Error>> {
        let home = std::env::var_os("HOME").ok_or("HOME is not set")?;
        let home = PathBuf::from(home);
        let karabiner = home.join(".config/karabiner");

        Ok(Paths {
            config: karabiner.join("karabiner.json"),
            asset: karabiner
                .join("assets/complex_modifications")
                .join(ASSET_FILE_NAME),
            env_file: home.join(".config/dedent-paste/env"),
        })
    }

    /// The path Karabiner should exec. Uses the path this process was started
    /// with (so Homebrew's stable `bin/` symlink is kept rather than the
    /// versioned Cellar directory) and normalises Cellar paths just in case.
    fn binary_path() -> Result<PathBuf, Box<dyn Error>> {
        let current = std::env::current_exe()?;
        let stable = stable_binary_path(&current);

        if !stable.is_file() {
            return Err(format!(
                "resolved executable path does not exist: {}",
                stable.display()
            )
            .into());
        }

        Ok(stable)
    }

    fn read_config(path: &Path) -> Result<Option<Value>, Box<dyn Error>> {
        if !path.exists() {
            return Ok(None);
        }

        let text = std::fs::read_to_string(path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        let value: Value = serde_json::from_str(&text)
            .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;

        Ok(Some(value))
    }

    fn write_config_with_backup(path: &Path, config: &Value) -> Result<PathBuf, Box<dyn Error>> {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|elapsed| format_timestamp(elapsed.as_secs()))
            .unwrap_or_default();
        let stamp: String = stamp.chars().filter(char::is_ascii_digit).collect();
        let mut backup = path.with_file_name(format!("karabiner.json.bak-{stamp}"));
        let mut counter = 1;
        while backup.exists() {
            backup = path.with_file_name(format!("karabiner.json.bak-{stamp}-{counter}"));
            counter += 1;
        }

        std::fs::copy(path, &backup)
            .map_err(|error| format!("failed to back up {}: {error}", path.display()))?;
        std::fs::write(path, to_karabiner_json(config))
            .map_err(|error| format!("failed to write {}: {error}", path.display()))?;

        Ok(backup)
    }

    fn lint_asset(asset: &Path) {
        let cli = if Path::new(KARABINER_CLI).is_file() {
            Some(PathBuf::from(KARABINER_CLI))
        } else {
            std::env::var_os("PATH").and_then(|path| {
                std::env::split_paths(&path)
                    .map(|dir| dir.join("karabiner_cli"))
                    .find(|candidate| candidate.is_file())
            })
        };

        let Some(cli) = cli else {
            return;
        };

        // Best effort: linting is a convenience, not a requirement.
        match Command::new(cli)
            .arg("--lint-complex-modifications")
            .arg(asset)
            .output()
        {
            Ok(output) if output.status.success() => {
                println!("Validated rule with karabiner_cli.");
            }
            Ok(output) => {
                eprintln!(
                    "warning: karabiner_cli reported a problem with the rule:\n{}{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            Err(_) => {}
        }
    }

    pub fn install() -> Result<(), Box<dyn Error>> {
        let paths = paths()?;
        let binary = binary_path()?;
        let (title, rule) = build_rule(&binary)?;

        if !Path::new(KARABINER_APP).exists() {
            eprintln!(
                "warning: Karabiner-Elements was not found at {KARABINER_APP}.\n         \
                 Install it with: brew install --cask karabiner-elements"
            );
        }

        if let Some(parent) = paths.asset.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
        }
        std::fs::write(&paths.asset, to_karabiner_json(&build_asset(&title, &rule)))
            .map_err(|error| format!("failed to write {}: {error}", paths.asset.display()))?;
        println!(
            "Wrote complex modification asset: {}",
            paths.asset.display()
        );

        match read_config(&paths.config)? {
            None => {
                println!(
                    "Karabiner profile not found at {}.\n\
                     Open Karabiner-Elements once, then run 'dedent-paste --install' again,\n\
                     or enable \"{title}\" manually under Complex Modifications > Add rule.",
                    paths.config.display()
                );
            }
            Some(mut config) => {
                let outcome = install_rule(&mut config, rule)?;
                let backup = write_config_with_backup(&paths.config, &config)?;

                println!("Backed up Karabiner profile: {}", backup.display());
                if outcome.replaced > 0 {
                    println!(
                        "Replaced {} existing dedent-paste rule(s) in profile \"{}\".",
                        outcome.replaced, outcome.profile_name
                    );
                } else {
                    println!(
                        "Added Option+V rule to profile \"{}\".",
                        outcome.profile_name
                    );
                }
            }
        }

        println!("Karabiner will run: {}", binary.display());
        lint_asset(&paths.asset);

        if !paths.env_file.exists() {
            println!(
                "\nOptional: to enable image-to-text (Gemini), create {} with your API key:\n  \
                 mkdir -p \"$HOME/.config/dedent-paste\"\n  \
                 printf 'export GEMINI_API_KEY=\"your-key\"\\n' > \"$HOME/.config/dedent-paste/env\"\n  \
                 chmod 600 \"$HOME/.config/dedent-paste/env\"",
                paths.env_file.display()
            );
        }

        println!(
            "\nDone. Press Option+V to run dedent-paste.\n\
             If nothing happens, allow Karabiner-Elements under\n\
             System Settings > Privacy & Security > Accessibility."
        );

        Ok(())
    }

    pub fn uninstall() -> Result<(), Box<dyn Error>> {
        let paths = paths()?;
        let mut changed = false;

        match read_config(&paths.config)? {
            None => println!(
                "Karabiner profile not found at {}; nothing to remove.",
                paths.config.display()
            ),
            Some(mut config) => {
                let outcome = uninstall_rule(&mut config)?;
                if outcome.total_removed() == 0 {
                    println!("No dedent-paste rule found in karabiner.json.");
                } else {
                    let backup = write_config_with_backup(&paths.config, &config)?;
                    println!("Backed up Karabiner profile: {}", backup.display());
                    for (profile, count) in &outcome.profiles {
                        println!(
                            "Removed {count} dedent-paste rule(s) from profile \"{profile}\"."
                        );
                    }
                    changed = true;
                }
            }
        }

        if paths.asset.is_file() {
            std::fs::remove_file(&paths.asset)
                .map_err(|error| format!("failed to remove {}: {error}", paths.asset.display()))?;
            println!(
                "Removed complex modification asset: {}",
                paths.asset.display()
            );
            changed = true;
        }

        if changed {
            println!("Done. The Option+V hotkey is no longer bound to dedent-paste.");
        } else {
            println!("Nothing to do.");
        }

        Ok(())
    }
}

#[cfg(not(target_os = "macos"))]
mod unsupported {
    use std::error::Error;

    const MESSAGE: &str = "--install and --uninstall manage Karabiner-Elements and are only available on macOS.\n\
        On Windows, bind Win+V with AutoHotkey; see https://github.com/doggy8088/dedent-paste#windows";

    pub fn install() -> Result<(), Box<dyn Error>> {
        Err(MESSAGE.into())
    }

    pub fn uninstall() -> Result<(), Box<dyn Error>> {
        Err(MESSAGE.into())
    }
}
