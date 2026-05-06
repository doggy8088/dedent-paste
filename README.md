# dedent-paste

`dedent-paste` is a macOS helper for Karabiner-Elements. It reads the current clipboard as plain text, removes the common leading space/tab prefix from non-blank lines, writes the transformed plain text back to the clipboard, then sends `Command+V`.

The intended hotkey is `Command+Option+Shift+V`.

## Build

```sh
cargo build --release
```

Karabiner is configured to call:

```text
/Users/will/projects/dedent-paste/target/release/dedent-paste
```

## Behavior

- Blank and whitespace-only lines are ignored when calculating the minimum indentation.
- Space and tab both count as one leading whitespace character.
- The transformed plain text remains in the clipboard after pasting.
- Paste is triggered with AppleScript/System Events, so macOS Accessibility permission may be required.

## Karabiner-Elements

The complex modification asset is installed at:

```text
~/.config/karabiner/assets/complex_modifications/paste-dedent-plain-text.json
```

The active Karabiner profile in `~/.config/karabiner/karabiner.json` also contains the same `Command+Option+Shift+V` rule.
