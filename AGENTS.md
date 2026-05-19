# AGENTS.md

Guidance for AI coding agents working in this repository.

## Project overview

`dedent-paste` is a Rust CLI that reads plain text from the clipboard, removes shared indentation from non-blank lines, writes the result back to the clipboard, and triggers paste.

Runtime support:

- macOS: uses `pbpaste`, `pbcopy`, and `osascript`; intended for Karabiner-Elements `Option+V`.
- Windows: uses native clipboard APIs plus simulated `Ctrl+V`; intended for AutoHotkey `Win+V`.
- Linux artifacts are published by cargo-dist, but runtime clipboard/paste integration is not documented.

## Repository layout

- `src/lib.rs`: pure dedent logic and unit tests.
- `src/main.rs`: platform clipboard and paste integrations.
- `README.md`: user-facing installation and usage docs, primarily Traditional Chinese.
- `DEVELOPMENT.md`: build, test, installer, CI/CD, and release notes.
- `install.sh`: macOS installer and Karabiner configuration updater.
- `examples/`: Karabiner and AutoHotkey integration examples.
- `docs/`: GitHub Pages static site.
- `.github/workflows/`: CI, Pages, and cargo-dist release workflows.

## Development commands

Run these before finishing code changes:

```sh
cargo fmt --check
cargo test --locked
cargo build --locked
```

For release-style local builds:

```sh
cargo build --release --locked
```

Do not introduce new build, lint, or test tools unless the task explicitly requires it.

## Coding guidelines

- Keep the dedent algorithm in `src/lib.rs` pure and covered by unit tests.
- Keep platform-specific clipboard and paste behavior behind `#[cfg(...)]` modules in `src/main.rs`.
- Preserve current error style: return `Result<_, Box<dyn Error>>` from platform/runtime paths and surface actionable error messages.
- Treat spaces and tabs as one indentation character each unless intentionally changing the documented behavior.
- Preserve newline behavior, including missing trailing newlines and CRLF handling.
- Avoid broad fallbacks that hide platform command or clipboard failures.
- Use `cargo fmt` formatting conventions.

## Testing expectations

- Add or update unit tests in `src/lib.rs` for dedent behavior changes.
- Prefer pure unit tests for text transformation logic; avoid tests that require a live clipboard, Karabiner-Elements, AutoHotkey, or GUI automation.
- When changing platform integration, consider macOS and Windows behavior separately and preserve unsupported-platform errors.

## Documentation expectations

- Update `README.md` when user-facing install, shortcut, or usage behavior changes.
- Update `DEVELOPMENT.md` when build, test, installer, CI/CD, or release behavior changes.
- Keep Traditional Chinese style in `README.md` unless editing an existing English-only section.
- Keep installer paths and shortcut examples consistent across README, examples, and installer code.

## Release and distribution notes

- Versioning is in `Cargo.toml`.
- cargo-dist configuration is in `dist-workspace.toml` and release workflows.
- The release workflow publishes shell and PowerShell installers plus platform archives.
- Be careful when changing installer URLs, asset names, or install locations because README, docs, examples, and workflows may all reference them.

## Commit guidelines

- Use Conventional Commits 1.0.0 for commit messages.
- Format the subject as `<type>[optional scope]: <description>`, for example `docs: update agent guidance`.
- Use `feat` for new user-facing behavior and `fix` for bug fixes. Use other Conventional Commit types such as `docs`, `test`, `refactor`, `build`, `ci`, or `chore` when appropriate.
- Mark breaking changes with `!` before the colon and include a `BREAKING CHANGE:` footer.
- Do not add blank lines inside the commit message body.
- Do not use multiple `git commit -m` arguments to write multiple lines of a commit message body.
- When a multi-line commit message is necessary, insert real newline characters. Do not write the literal string `\n` into the git log message.

## Safety notes

- Do not overwrite user Karabiner or AutoHotkey configuration in examples or installer changes without preserving the existing backup/validation behavior.
- Do not commit generated build output from `target/`.
- Do not add secrets, tokens, machine-specific paths, or local editor configuration.
