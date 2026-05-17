# Changelog

All notable changes to this project are documented in this file.

## Unreleased

## 0.2.1

- Added support for stripping common terminal prompt prefixes when normalizing pasted text.

## 0.2.0

- Added Windows runtime support with native clipboard handling and simulated `Ctrl+V` paste.
- Added AutoHotkey v1/v2 examples for using `dedent-paste` with `Win+V` on Windows.
- Updated `README.md` with separate macOS and Windows installation, setup, and usage guides.
- Documented the PowerShell Release installer and clarified `PATH` vs fixed-path AutoHotkey setup.
- Updated `docs/index.html` and `DEVELOPMENT.md` to reflect macOS + Windows support.

## 0.1.1

- Simplified `README.md` for end users.
- Added `DEVELOPMENT.md` for build, install, Karabiner, and CI/CD details.
- Added this changelog.
- Updated GitHub Actions to publish cargo-dist release installers and platform archives.
- Updated GitHub Actions to dispatch cargo-dist releases only when `Cargo.toml` package version changes.
- Added GitHub Pages landing page and deployment workflow.
- Optimized the GitHub Pages landing page with small WebP visual assets.

## 0.1.0

- Added `dedent-paste` command-line helper for Karabiner-Elements.
- Added `Option+V` Karabiner-Elements integration.
- Added UTF-8 clipboard handling for Karabiner `shell_command` execution.
- Added one-line installer.
- Added MIT license.
