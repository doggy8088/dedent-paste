#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_dir="$script_dir"
install_dir="${DEDENT_PASTE_INSTALL_DIR:-$HOME/.local/bin}"
installer_url="https://github.com/doggy8088/dedent-paste/releases/latest/download/dedent-paste-installer.sh"

if [[ -f "$repo_dir/Cargo.toml" ]]; then
  if ! command -v cargo >/dev/null 2>&1; then
    echo "error: cargo is required to build dedent-paste from source" >&2
    exit 1
  fi

  cargo build --release --manifest-path "$repo_dir/Cargo.toml"
  binary_path="$repo_dir/target/release/dedent-paste"
else
  if ! command -v curl >/dev/null 2>&1; then
    echo "error: curl is required to download dedent-paste" >&2
    exit 1
  fi

  mkdir -p "$install_dir"
  binary_path="$install_dir/dedent-paste"
  curl --fail --location --retry 3 --silent --show-error "$installer_url" \
    | DEDENT_PASTE_INSTALL_DIR="$install_dir" INSTALLER_NO_MODIFY_PATH=1 sh -s -- --quiet

  if [[ ! -x "$binary_path" ]]; then
    echo "error: cargo-dist installer did not create $binary_path" >&2
    exit 1
  fi
fi

# Karabiner-Elements configuration (asset file, profile backup, rule install,
# karabiner_cli lint) is implemented by the binary itself so every install
# channel (this script, Homebrew, npm) behaves the same way.
#
# Releases before 0.3.3 ignore command-line flags and would paste the clipboard
# instead, so refuse to continue with a binary that lacks the option parser.
if ! grep -a -q -- 'Usage: dedent-paste' "$binary_path"; then
  echo "error: $binary_path predates the --install flag (needs dedent-paste >= 0.3.3)." >&2
  echo "       Re-run this installer after a newer release is published." >&2
  exit 1
fi

"$binary_path" --install
