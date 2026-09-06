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

Because releases before 0.4.0 ignore command-line arguments and would run the paste flow instead, `install.sh` refuses to call `--install` on a binary that does not contain the help text.

## Self-update behavior (`dedent-paste update`)

`dedent-paste update` allows standalone installations to update themselves to the latest published release.

The update flow:

1. Resolves `current_exe()` and classifies the install method (`update::detect_install_method`):
   - Homebrew (`Cellar`, `/opt/homebrew/`, `/.linuxbrew/`)
   - npm (`node_modules`, `/.nvm/`, `/npm/`, `/npm-global/`)
   - Cargo (`.cargo/bin`)
   - Local build (`target/debug/`, `target/release/`)
   - Standalone binary (e.g. `~/.local/bin/dedent-paste`)
2. Fetches the latest release version from `dist-manifest.json` (downloaded from GitHub Releases; no GitHub API rate limits), with a fallback to the GitHub Releases API.
3. Compares SemVer versions (`update::is_newer_version`).
4. If `--check` (or `-c`) is passed: reports whether an update is available along with the recommended command for the detected install method, then exits 0.
5. If the current version is already up to date and `--force` (or `-f`) is not set: prints that the tool is up to date and exits 0.
6. If the installation was managed by a package manager (Homebrew, npm, Cargo) or is a local build, returns an actionable error directing the user to use the proper tool or skipping the update.
7. For standalone installations:
   - macOS / Linux: downloads the latest `dedent-paste-installer.sh` using `ureq` and pipes it to `sh -s -- --quiet` with `DEDENT_PASTE_INSTALL_DIR` set to the binary's directory and `INSTALLER_NO_MODIFY_PATH=1`. On macOS, it re-runs `--install` on the new binary so Karabiner-Elements rules stay in sync.
   - Windows: renames the running `.exe` to `.exe.old`, downloads `dedent-paste-installer.ps1`, and invokes PowerShell with `DEDENT_PASTE_INSTALL_DIR` set. If the installer fails, it restores `.exe.old`; if it succeeds, it removes `.exe.old`.

Pure SemVer, manifest parsing, and install-method detection live in `src/update.rs` with unit tests.

## Karabiner rule

```json
{
  "description": "Option+V：執行 dedent-paste",
  "manipulators": [
    {
      "from": {
        "key_code": "v",
        "modifiers": { "mandatory": ["left_option"] }
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

## Paste sequence (macOS)

`run()` in `src/main.rs` performs, in order:

1. Resolve `PasteSettings` (`lib.rs::resolve_paste_settings`) from `--no-paste` / `--paste-delay-ms` and the `DEDENT_PASTE_NO_PASTE` / `DEDENT_PASTE_PASTE_DELAY_MS` environment variables. Flags win.
2. Take a non-blocking exclusive `flock` on `$TMPDIR/dedent-paste.lock` (`platform::try_lock_single_instance`). If another instance holds it, log one info line and exit 0. This stops key-repeating hotkey managers (skhd) from spawning many pasting processes.
3. Read, dedent, and write the clipboard (or run the Gemini image path).
4. `finish_paste`: return early on `--no-paste`; otherwise `platform::wait_for_modifiers_released` polls `CGEventSourceFlagsState(kCGEventSourceStateCombinedSessionState)` every 10 ms until Shift/Control/Option/Command are all clear or 1 s has passed, then sleeps for the configured delay, then sends `Command+V` via `osascript`.

`CGEventSourceFlagsState` and `flock` are declared with raw `extern "C"` blocks (CoreGraphics framework and libSystem respectively) to avoid adding binding crates. Reading modifier state needs no Accessibility or Input Monitoring permission.

Why the wait matters: when Karabiner runs the `shell_command`, the physical Option key is usually still down. A `Command+V` posted at that moment reaches the target app as `Command+Option+V`, which most apps ignore or treat as "paste and match style", even though `osascript` exits 0. See issue #1.

Windows and other platforms implement `wait_for_modifiers_released` and `try_lock_single_instance` as no-ops: the documented AutoHotkey scripts already `KeyWait` for the Win keys and use `RunWait`, and AutoHotkey runs one hotkey thread at a time.

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

cargo-dist has no support for Homebrew `caveats`, so the tap repo has its own workflow (`.github/workflows/caveats.yml`) that runs `scripts/add-caveats.py` after every formula push. It appends a `caveats` block reminding users to run `dedent-paste --install` for Karabiner-Elements, and skips formulas older than 0.4.0 because those binaries do not understand the flag. Edit the caveats text in that script, not in the formula.

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
