# Development

## Build

```sh
cargo build --release
```

When `./install.sh` is run from a source checkout, it builds the local source and configures Karabiner-Elements to call:

```text
<repo>/target/release/dedent-paste
```

When `install.sh` is run from the one-line installer in `README.md`, it downloads the latest release executable and configures Karabiner-Elements to call:

```text
$HOME/.local/bin/dedent-paste
```

## Test

```sh
cargo fmt --check
cargo test --locked
cargo build --release --locked
```

## Installer behavior

`install.sh` requires Python 3 to update Karabiner JSON files.

The installer writes this complex modification asset:

```text
~/.config/karabiner/assets/complex_modifications/paste-dedent-plain-text.json
```

It also backs up the active Karabiner configuration before modifying it:

```text
~/.config/karabiner/karabiner.json.bak-YYYYMMDDHHMMSS
```

If `karabiner_cli` is available, the installer validates the generated complex modification JSON.

## Karabiner rule

```json
{
  "description": "Option+V：執行 dedent-paste",
  "manipulators": [
    {
      "from": {
        "key_code": "v",
        "modifiers": { "mandatory": ["option"] }
      },
      "to": [
        {
          "shell_command": "$HOME/.local/bin/dedent-paste"
        }
      ],
      "type": "basic"
    }
  ]
}
```

## CI/CD

GitHub Actions runs on every push to `main`.

The workflow:

1. Checks out the repository.
2. Installs stable Rust.
3. Checks formatting.
4. Runs tests.
5. Builds a universal macOS executable for Apple Silicon and Intel Macs.
6. Creates a GitHub Release tagged as `main-<short-sha>`.
7. Uploads one release asset named `dedent-paste`.

The one-line installer downloads:

```text
https://github.com/doggy8088/dedent-paste/releases/latest/download/dedent-paste
```
