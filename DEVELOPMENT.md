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

`install.sh` installs (or builds) the binary and then runs `dedent-paste --install`, so the Karabiner-Elements logic lives in one place for every install channel (installer script, Homebrew, npm). Python 3 is no longer required.

`dedent-paste --install`:

1. Determines its own path with `std::env::current_exe()`, keeping the invoked path (for Homebrew this is the stable `bin/` symlink) and rewriting `Cellar/<formula>/<version>/` paths to the version-independent `opt/<formula>/` path (`karabiner::stable_binary_path`).
2. Builds the rule from `examples/macos/paste-dedent-plain-text.json`, which is embedded at compile time with `include_str!`, and rewrites every `shell_command` to point at that path. Paths with unsafe characters are single-quoted.
3. Writes the complex modification asset:

   ```text
   ~/.config/karabiner/assets/complex_modifications/paste-dedent-plain-text.json
   ```

4. Backs up the active Karabiner configuration before modifying it:

   ```text
   ~/.config/karabiner/karabiner.json.bak-YYYYMMDDHHMMSS
   ```

5. Installs the rule into the selected profile. Existing rules are matched by whether any manipulator's `shell_command` contains `dedent-paste` (not by description); the first match is replaced in place and further matches are removed.
6. Lints the asset with `karabiner_cli` if it is available.

`dedent-paste --uninstall` removes matching rules from every profile (with the same backup) and deletes the asset file.

Both flags are macOS-only and return an error on other platforms. The pure JSON manipulation is in `src/karabiner.rs` and is unit tested; file-system access is in `src/setup.rs`.

Because releases before 0.3.3 ignore command-line arguments and would run the paste flow instead, `install.sh` refuses to call `--install` on a binary that does not contain the help text.

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
6. The `publish-homebrew-formula` job pushes the generated `Formula/dedent-paste.rb` to the Homebrew tap repo `doggy8088/homebrew-dedent-paste`. It authenticates with the `HOMEBREW_TAP_TOKEN` repository secret, which must be a GitHub token with `contents: write` access to the tap repo.

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

## Dependencies

The Gemini image-to-text feature adds three runtime crates:

- `ureq` (blocking HTTP; rustls + bundled webpki roots — no OpenSSL, so it cross-compiles cleanly for all cargo-dist targets)
- `serde_json` (request/response JSON with correct escaping of user-provided prompts)
- `base64` (inline image payload encoding)

Windows-only, `image` (with only the `bmp` and `png` features) converts CF_DIB clipboard bitmaps to PNG. Clipboard image reading on macOS shells out to `osascript` and needs no extra crate. Keep new dependencies minimal and pure-Rust so the five cargo-dist release targets keep building.

## Homebrew tap

The Homebrew tap lives in a separate repository, <https://github.com/doggy8088/homebrew-dedent-paste>, and is checked out here as the `homebrew-tap/` git submodule for reference:

```sh
git submodule update --init
```

The formula is generated by cargo-dist (`installers` includes `homebrew` and `tap` is set in `dist-workspace.toml`) and committed to the tap by the release workflow. Do not edit `homebrew-tap/Formula/dedent-paste.rb` by hand; change `dist-workspace.toml` or the `description` / `homepage` fields in `Cargo.toml` and let the next release regenerate it.

cargo-dist has no support for Homebrew `caveats`, so the tap repo has its own workflow (`.github/workflows/caveats.yml`) that runs `scripts/add-caveats.py` after every formula push. It appends a `caveats` block reminding users to run `dedent-paste --install` for Karabiner-Elements, and skips formulas older than 0.3.3 because those binaries do not understand the flag. Edit the caveats text in that script, not in the formula.

To preview the formula locally run:

```sh
dist build --artifacts=global
cat target/distrib/dedent-paste.rb
```

The local preview has no `sha256` lines because the release archives are not present; CI fills them in.

After a release, refresh the submodule pointer with:

```sh
git submodule update --remote homebrew-tap
git add homebrew-tap
```

## GitHub Pages

The GitHub Pages site is in:

```text
public/
```

The `GitHub Pages` workflow deploys the static site on pushes to `main` that touch `public/**`. Use the workflow's `Run workflow` button (`workflow_dispatch`) to redeploy without a content change.
