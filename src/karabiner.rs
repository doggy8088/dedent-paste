//! Pure logic for installing and removing the Karabiner-Elements rule that
//! binds `Option+V` to `dedent-paste`.
//!
//! Everything here operates on in-memory JSON so it can be unit tested. File
//! system access, backups, and `karabiner_cli` linting live in `main.rs`.

use std::error::Error;
use std::fmt;
use std::path::{Component, Path, PathBuf};

use serde_json::{Map, Value, json};

/// The rule template shipped in `examples/macos/paste-dedent-plain-text.json`.
/// The `shell_command` inside it is rewritten to the actual install path.
pub const RULE_TEMPLATE: &str = include_str!("../examples/macos/paste-dedent-plain-text.json");

/// Substring used to recognise an existing dedent-paste rule by what it runs,
/// rather than by its human-readable description.
pub const BINARY_KEYWORD: &str = "dedent-paste";

/// Environment file sourced before `dedent-paste` runs so that
/// `GEMINI_API_KEY` and friends are visible when Karabiner triggers the rule.
pub const ENV_FILE_SHELL_PATH: &str = "$HOME/.config/dedent-paste/env";

/// File name of the complex modification asset written for manual import.
pub const ASSET_FILE_NAME: &str = "paste-dedent-plain-text.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KarabinerError {
    InvalidTemplate(String),
    InvalidConfig(String),
    NoProfiles,
    NonUtf8Path(PathBuf),
}

impl fmt::Display for KarabinerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KarabinerError::InvalidTemplate(detail) => {
                write!(f, "built-in Karabiner rule template is invalid: {detail}")
            }
            KarabinerError::InvalidConfig(detail) => {
                write!(f, "karabiner.json has an unexpected structure: {detail}")
            }
            KarabinerError::NoProfiles => {
                write!(f, "karabiner.json does not contain any profiles")
            }
            KarabinerError::NonUtf8Path(path) => {
                write!(f, "executable path is not valid UTF-8: {}", path.display())
            }
        }
    }
}

impl Error for KarabinerError {}

/// Result of installing the rule into a profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallOutcome {
    pub profile_name: String,
    /// Number of pre-existing dedent-paste rules that were replaced.
    pub replaced: usize,
}

/// Result of removing the rule from every profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UninstallOutcome {
    /// `(profile name, rules removed)` for each profile that changed.
    pub profiles: Vec<(String, usize)>,
}

impl UninstallOutcome {
    pub fn total_removed(&self) -> usize {
        self.profiles.iter().map(|(_, count)| count).sum()
    }
}

/// Map a Homebrew `Cellar` path to the version-independent `opt` path so the
/// rule survives `brew upgrade`. Other paths are returned unchanged.
///
/// `/opt/homebrew/Cellar/dedent-paste/0.3.2/bin/dedent-paste` becomes
/// `/opt/homebrew/opt/dedent-paste/bin/dedent-paste`.
pub fn stable_binary_path(path: &Path) -> PathBuf {
    let components: Vec<Component<'_>> = path.components().collect();
    let cellar_index = components
        .iter()
        .position(|component| matches!(component, Component::Normal(name) if *name == "Cellar"));

    let Some(cellar_index) = cellar_index else {
        return path.to_path_buf();
    };

    // Expect: <prefix>/Cellar/<formula>/<version>/<rest...>
    let Some(formula) = components.get(cellar_index + 1) else {
        return path.to_path_buf();
    };
    if components.len() <= cellar_index + 3 {
        return path.to_path_buf();
    }

    let mut stable = PathBuf::new();
    for component in &components[..cellar_index] {
        stable.push(component.as_os_str());
    }
    stable.push("opt");
    stable.push(formula.as_os_str());
    for component in &components[cellar_index + 3..] {
        stable.push(component.as_os_str());
    }

    stable
}

/// Quote a path for use inside a `sh -c` command. Paths made only of safe
/// characters are returned as-is so the common case stays readable.
pub fn shell_quote(text: &str) -> String {
    let is_safe = |c: char| c.is_ascii_alphanumeric() || "/._-+:@%".contains(c);

    if !text.is_empty() && text.chars().all(is_safe) {
        return text.to_string();
    }

    format!("'{}'", text.replace('\'', r"'\''"))
}

/// Build the `shell_command` that Karabiner runs for the rule.
pub fn build_shell_command(binary_path: &Path) -> Result<String, KarabinerError> {
    let path = binary_path
        .to_str()
        .ok_or_else(|| KarabinerError::NonUtf8Path(binary_path.to_path_buf()))?;

    Ok(format!(
        ". \"{ENV_FILE_SHELL_PATH}\" 2>/dev/null; exec {}",
        shell_quote(path)
    ))
}

/// Parse the bundled template and return `(title, rule)` with every
/// `shell_command` pointed at `binary_path`.
pub fn build_rule(binary_path: &Path) -> Result<(String, Value), KarabinerError> {
    let template: Value = serde_json::from_str(RULE_TEMPLATE)
        .map_err(|error| KarabinerError::InvalidTemplate(error.to_string()))?;

    let title = template
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("Dedent paste")
        .to_string();

    let mut rule = template
        .get("rules")
        .and_then(Value::as_array)
        .and_then(|rules| rules.first())
        .cloned()
        .ok_or_else(|| KarabinerError::InvalidTemplate("missing rules[0]".into()))?;

    let shell_command = build_shell_command(binary_path)?;
    let manipulators = rule
        .get_mut("manipulators")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| KarabinerError::InvalidTemplate("missing manipulators".into()))?;

    let mut rewritten = 0;
    for manipulator in manipulators.iter_mut() {
        for event in to_events_mut(manipulator) {
            if let Some(object) = event.as_object_mut() {
                if object.contains_key("shell_command") {
                    object.insert("shell_command".into(), Value::String(shell_command.clone()));
                    rewritten += 1;
                }
            }
        }
    }

    if rewritten == 0 {
        return Err(KarabinerError::InvalidTemplate(
            "no shell_command found in template".into(),
        ));
    }

    Ok((title, rule))
}

/// Build the standalone complex-modification asset file content.
pub fn build_asset(title: &str, rule: &Value) -> Value {
    json!({
        "title": title,
        "rules": [rule],
    })
}

/// A rule belongs to dedent-paste when any of its manipulators runs a
/// `shell_command` that mentions the binary. The description is deliberately
/// ignored so renamed or hand-edited rules are still recognised.
pub fn rule_targets_dedent_paste(rule: &Value) -> bool {
    let Some(manipulators) = rule.get("manipulators").and_then(Value::as_array) else {
        return false;
    };

    manipulators.iter().any(|manipulator| {
        to_events(manipulator).iter().any(|event| {
            event
                .get("shell_command")
                .and_then(Value::as_str)
                .is_some_and(|command| command.contains(BINARY_KEYWORD))
        })
    })
}

/// Install `rule` into the selected profile of `config` (Karabiner's
/// `karabiner.json`). Existing dedent-paste rules are replaced in place; when
/// none exist the rule is appended.
pub fn install_rule(config: &mut Value, rule: Value) -> Result<InstallOutcome, KarabinerError> {
    let profile = selected_profile_mut(config)?;
    let profile_name = profile_name(profile);
    let rules = rules_mut(profile)?;

    let matching: Vec<usize> = rules
        .iter()
        .enumerate()
        .filter(|(_, existing)| rule_targets_dedent_paste(existing))
        .map(|(index, _)| index)
        .collect();

    match matching.split_first() {
        None => rules.push(rule),
        Some((first, rest)) => {
            rules[*first] = rule;
            for index in rest.iter().rev() {
                rules.remove(*index);
            }
        }
    }

    Ok(InstallOutcome {
        profile_name,
        replaced: matching.len(),
    })
}

/// Remove every dedent-paste rule from every profile in `config`.
pub fn uninstall_rule(config: &mut Value) -> Result<UninstallOutcome, KarabinerError> {
    let profiles = config
        .get_mut("profiles")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| KarabinerError::InvalidConfig("profiles is not an array".into()))?;

    let mut changed = Vec::new();
    for profile in profiles.iter_mut() {
        let name = profile_name(profile);
        let Some(rules) = profile
            .get_mut("complex_modifications")
            .and_then(|cm| cm.get_mut("rules"))
            .and_then(Value::as_array_mut)
        else {
            continue;
        };

        let before = rules.len();
        rules.retain(|rule| !rule_targets_dedent_paste(rule));
        let removed = before - rules.len();
        if removed > 0 {
            changed.push((name, removed));
        }
    }

    Ok(UninstallOutcome { profiles: changed })
}

/// Serialise with the four-space indentation Karabiner-Elements itself uses.
pub fn to_karabiner_json(value: &Value) -> String {
    let two_space = serde_json::to_string_pretty(value).unwrap_or_default();
    let mut out = String::with_capacity(two_space.len() + two_space.len() / 4);

    for line in two_space.lines() {
        let indent = line.len() - line.trim_start_matches(' ').len();
        out.extend(std::iter::repeat_n(' ', indent * 2));
        out.push_str(&line[indent..]);
        out.push('\n');
    }

    out
}

fn selected_profile_mut(config: &mut Value) -> Result<&mut Value, KarabinerError> {
    let profiles = config
        .get_mut("profiles")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| KarabinerError::InvalidConfig("profiles is not an array".into()))?;

    if profiles.is_empty() {
        return Err(KarabinerError::NoProfiles);
    }

    let index = profiles
        .iter()
        .position(|profile| profile.get("selected").and_then(Value::as_bool) == Some(true))
        .unwrap_or(0);

    Ok(&mut profiles[index])
}

fn profile_name(profile: &Value) -> String {
    profile
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("(unnamed)")
        .to_string()
}

fn rules_mut(profile: &mut Value) -> Result<&mut Vec<Value>, KarabinerError> {
    let profile = profile
        .as_object_mut()
        .ok_or_else(|| KarabinerError::InvalidConfig("profile is not an object".into()))?;

    let complex = profile
        .entry("complex_modifications")
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .ok_or_else(|| {
            KarabinerError::InvalidConfig("complex_modifications is not an object".into())
        })?;

    complex
        .entry("rules")
        .or_insert_with(|| Value::Array(Vec::new()))
        .as_array_mut()
        .ok_or_else(|| {
            KarabinerError::InvalidConfig("complex_modifications.rules is not an array".into())
        })
}

/// Karabiner accepts `to` as either a single object or an array of objects.
fn to_events(manipulator: &Value) -> Vec<&Value> {
    match manipulator.get("to") {
        Some(Value::Array(events)) => events.iter().collect(),
        Some(event @ Value::Object(_)) => vec![event],
        _ => Vec::new(),
    }
}

fn to_events_mut(manipulator: &mut Value) -> Vec<&mut Value> {
    match manipulator.get_mut("to") {
        Some(Value::Array(events)) => events.iter_mut().collect(),
        Some(event @ Value::Object(_)) => vec![event],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_with_rules(selected_rules: Vec<Value>, other_rules: Vec<Value>) -> Value {
        json!({
            "profiles": [
                {
                    "name": "Work",
                    "selected": false,
                    "complex_modifications": { "rules": other_rules }
                },
                {
                    "name": "Default profile",
                    "selected": true,
                    "complex_modifications": { "rules": selected_rules }
                }
            ]
        })
    }

    fn shell_rule(description: &str, command: &str) -> Value {
        json!({
            "description": description,
            "manipulators": [{
                "type": "basic",
                "from": { "key_code": "v", "modifiers": { "mandatory": ["option"] } },
                "to": [{ "shell_command": command }]
            }]
        })
    }

    fn rule_commands(config: &Value, profile_index: usize) -> Vec<String> {
        config["profiles"][profile_index]["complex_modifications"]["rules"]
            .as_array()
            .unwrap()
            .iter()
            .map(|rule| {
                rule["manipulators"][0]["to"][0]["shell_command"]
                    .as_str()
                    .unwrap_or("")
                    .to_string()
            })
            .collect()
    }

    #[test]
    fn template_parses_and_points_at_install_path() {
        let (title, rule) = build_rule(Path::new("/opt/homebrew/bin/dedent-paste")).unwrap();

        assert_eq!(title, "Dedent paste");
        assert_eq!(
            rule["manipulators"][0]["to"][0]["shell_command"],
            ". \"$HOME/.config/dedent-paste/env\" 2>/dev/null; exec /opt/homebrew/bin/dedent-paste"
        );
        assert_eq!(rule["manipulators"][0]["from"]["key_code"], "v");
        assert_eq!(
            rule["manipulators"][0]["from"]["modifiers"]["mandatory"],
            json!(["left_option"])
        );
        assert!(rule_targets_dedent_paste(&rule));
    }

    #[test]
    fn asset_wraps_rule_with_title() {
        let (title, rule) = build_rule(Path::new("/usr/local/bin/dedent-paste")).unwrap();
        let asset = build_asset(&title, &rule);

        assert_eq!(asset["title"], "Dedent paste");
        assert_eq!(asset["rules"].as_array().unwrap().len(), 1);
        assert_eq!(asset["rules"][0], rule);
    }

    #[test]
    fn shell_command_quotes_paths_with_spaces() {
        let command = build_shell_command(Path::new("/Users/Some One/bin/dedent-paste")).unwrap();
        assert!(command.ends_with("exec '/Users/Some One/bin/dedent-paste'"));

        let command = build_shell_command(Path::new("/it's/dedent-paste")).unwrap();
        assert!(command.ends_with(r"exec '/it'\''s/dedent-paste'"));
    }

    #[test]
    fn shell_quote_leaves_safe_paths_alone() {
        assert_eq!(
            shell_quote("/opt/homebrew/bin/dedent-paste"),
            "/opt/homebrew/bin/dedent-paste"
        );
        assert_eq!(shell_quote("a b"), "'a b'");
        assert_eq!(shell_quote("$HOME/x"), "'$HOME/x'");
        assert_eq!(shell_quote(""), "''");
    }

    #[test]
    fn cellar_paths_become_opt_paths() {
        assert_eq!(
            stable_binary_path(Path::new(
                "/opt/homebrew/Cellar/dedent-paste/0.3.2/bin/dedent-paste"
            )),
            PathBuf::from("/opt/homebrew/opt/dedent-paste/bin/dedent-paste")
        );
        assert_eq!(
            stable_binary_path(Path::new(
                "/usr/local/Cellar/dedent-paste/0.3.2_1/bin/dedent-paste"
            )),
            PathBuf::from("/usr/local/opt/dedent-paste/bin/dedent-paste")
        );
        assert_eq!(
            stable_binary_path(Path::new("/opt/homebrew/bin/dedent-paste")),
            PathBuf::from("/opt/homebrew/bin/dedent-paste")
        );
        assert_eq!(
            stable_binary_path(Path::new("/Users/me/.local/bin/dedent-paste")),
            PathBuf::from("/Users/me/.local/bin/dedent-paste")
        );
        // Too short to be a Cellar layout: leave untouched.
        assert_eq!(
            stable_binary_path(Path::new("/x/Cellar/dedent-paste")),
            PathBuf::from("/x/Cellar/dedent-paste")
        );
    }

    #[test]
    fn detection_uses_shell_command_not_description() {
        let renamed = shell_rule("My paste hotkey", "exec /somewhere/dedent-paste");
        assert!(rule_targets_dedent_paste(&renamed));

        let impostor = shell_rule("Option+V：執行 dedent-paste", "open -a Finder");
        assert!(!rule_targets_dedent_paste(&impostor));

        let no_shell = json!({
            "description": "Caps to Esc",
            "manipulators": [{
                "type": "basic",
                "from": { "key_code": "caps_lock" },
                "to": [{ "key_code": "escape" }]
            }]
        });
        assert!(!rule_targets_dedent_paste(&no_shell));
    }

    #[test]
    fn detection_accepts_single_object_to() {
        let rule = json!({
            "description": "x",
            "manipulators": [{
                "type": "basic",
                "from": { "key_code": "v" },
                "to": { "shell_command": "$HOME/.local/bin/dedent-paste" }
            }]
        });
        assert!(rule_targets_dedent_paste(&rule));
    }

    #[test]
    fn install_appends_when_no_rule_exists() {
        let mut config = config_with_rules(vec![shell_rule("Other", "open -a Finder")], vec![]);
        let (_, rule) = build_rule(Path::new("/opt/homebrew/bin/dedent-paste")).unwrap();

        let outcome = install_rule(&mut config, rule).unwrap();

        assert_eq!(outcome.profile_name, "Default profile");
        assert_eq!(outcome.replaced, 0);
        let commands = rule_commands(&config, 1);
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0], "open -a Finder");
        assert!(commands[1].ends_with("exec /opt/homebrew/bin/dedent-paste"));
        // Non-selected profile untouched.
        assert!(rule_commands(&config, 0).is_empty());
    }

    #[test]
    fn install_replaces_existing_rule_in_place_and_drops_duplicates() {
        let mut config = config_with_rules(
            vec![
                shell_rule(
                    "Old",
                    "exec /Users/me/projects/dedent-paste/target/release/dedent-paste",
                ),
                shell_rule("Other", "open -a Finder"),
                shell_rule("Stale duplicate", "$HOME/.local/bin/dedent-paste"),
            ],
            vec![],
        );
        let (_, rule) = build_rule(Path::new("/opt/homebrew/bin/dedent-paste")).unwrap();

        let outcome = install_rule(&mut config, rule).unwrap();

        assert_eq!(outcome.replaced, 2);
        let commands = rule_commands(&config, 1);
        assert_eq!(commands.len(), 2);
        assert!(commands[0].ends_with("exec /opt/homebrew/bin/dedent-paste"));
        assert_eq!(commands[1], "open -a Finder");
    }

    #[test]
    fn install_creates_missing_complex_modifications() {
        let mut config = json!({ "profiles": [{ "name": "Fresh", "selected": true }] });
        let (_, rule) = build_rule(Path::new("/opt/homebrew/bin/dedent-paste")).unwrap();

        let outcome = install_rule(&mut config, rule).unwrap();

        assert_eq!(outcome.profile_name, "Fresh");
        assert_eq!(rule_commands(&config, 0).len(), 1);
    }

    #[test]
    fn install_falls_back_to_first_profile_when_none_selected() {
        let mut config = json!({ "profiles": [{ "name": "A" }, { "name": "B" }] });
        let (_, rule) = build_rule(Path::new("/opt/homebrew/bin/dedent-paste")).unwrap();

        let outcome = install_rule(&mut config, rule).unwrap();

        assert_eq!(outcome.profile_name, "A");
    }

    #[test]
    fn install_rejects_configs_without_profiles() {
        let (_, rule) = build_rule(Path::new("/opt/homebrew/bin/dedent-paste")).unwrap();

        assert_eq!(
            install_rule(&mut json!({ "profiles": [] }), rule.clone()),
            Err(KarabinerError::NoProfiles)
        );
        assert!(matches!(
            install_rule(&mut json!({ "global": {} }), rule),
            Err(KarabinerError::InvalidConfig(_))
        ));
    }

    #[test]
    fn uninstall_removes_from_every_profile() {
        let mut config = config_with_rules(
            vec![
                shell_rule("Other", "open -a Finder"),
                shell_rule("Renamed", "exec /opt/homebrew/bin/dedent-paste"),
            ],
            vec![shell_rule("Old", "$HOME/.local/bin/dedent-paste")],
        );

        let outcome = uninstall_rule(&mut config).unwrap();

        assert_eq!(outcome.total_removed(), 2);
        assert_eq!(
            outcome.profiles,
            vec![("Work".to_string(), 1), ("Default profile".to_string(), 1)]
        );
        assert_eq!(rule_commands(&config, 0), Vec::<String>::new());
        assert_eq!(
            rule_commands(&config, 1),
            vec!["open -a Finder".to_string()]
        );
    }

    #[test]
    fn uninstall_is_a_no_op_without_matching_rules() {
        let mut config = config_with_rules(vec![shell_rule("Other", "open -a Finder")], vec![]);
        let before = config.clone();

        let outcome = uninstall_rule(&mut config).unwrap();

        assert_eq!(outcome.total_removed(), 0);
        assert!(outcome.profiles.is_empty());
        assert_eq!(config, before);
    }

    #[test]
    fn karabiner_json_uses_four_space_indent() {
        let value = json!({ "a": { "b": [1, "  keep leading spaces"] } });
        let text = to_karabiner_json(&value);

        assert_eq!(
            text,
            "{\n    \"a\": {\n        \"b\": [\n            1,\n            \"  keep leading spaces\"\n        ]\n    }\n}\n"
        );
        assert_eq!(serde_json::from_str::<Value>(&text).unwrap(), value);
    }
}
