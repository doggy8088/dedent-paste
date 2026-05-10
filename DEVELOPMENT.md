# Development

## Build

```sh
cargo build --release
```

When `./install.sh` is run from a source checkout, it builds the local source and configures Karabiner-Elements to call:

```text
<repo>/target/release/dedent-paste
```

When `install.sh` is run from the one-line installer in `README.md`, it runs the latest cargo-dist shell installer and configures Karabiner-Elements to call:

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

GitHub Actions runs CI on every push to `main` and every pull request.

The CI workflow:

1. Checks out the repository.
2. Installs stable Rust.
3. Checks formatting.
4. Runs tests.
5. Builds the project.

The release flow uses cargo-dist:

1. `.github/workflows/version-release.yml` runs on pushes to `main` that modify `Cargo.toml`.
2. It compares the current package version with the previous pushed commit.
3. If the version changed and release `v<version>` does not already exist, it dispatches `.github/workflows/release.yml`.
4. The cargo-dist `Release` workflow builds archives and checksums for configured targets:
   - `aarch64-apple-darwin`
   - `x86_64-apple-darwin`
   - `aarch64-unknown-linux-gnu`
   - `x86_64-unknown-linux-gnu`
   - `x86_64-pc-windows-msvc`
5. The cargo-dist workflow uploads shell and PowerShell installers plus platform archives to GitHub Releases.

The one-line macOS installer downloads and runs:

```text
https://github.com/doggy8088/dedent-paste/releases/latest/download/dedent-paste-installer.sh
```

The corresponding Windows PowerShell installer is:

```text
https://github.com/doggy8088/dedent-paste/releases/latest/download/dedent-paste-installer.ps1
```

`dedent-paste` supports macOS and Windows at runtime:

- macOS uses `pbpaste`, `pbcopy`, and `osascript`, and is intended to be triggered by Karabiner-Elements.
- Windows uses native clipboard APIs and simulated `Ctrl+V`, and the hotkey integration in `README.md` uses AutoHotkey.
- Linux artifacts are still published by cargo-dist for consistency, but runtime clipboard/paste integration is not documented.

## GitHub Pages

The GitHub Pages site is in:

```text
docs/
```

The `GitHub Pages` workflow deploys the static site on every push to `main`.
