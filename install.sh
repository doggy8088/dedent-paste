#!/usr/bin/env bash
set -euo pipefail

repo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
description="Option+V：執行 dedent-paste"
asset_path="$HOME/.config/karabiner/assets/complex_modifications/paste-dedent-plain-text.json"

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo is required to build dedent-paste" >&2
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "error: python3 is required to update Karabiner JSON files" >&2
  exit 1
fi

cargo build --release --manifest-path "$repo_dir/Cargo.toml"

REPO_DIR="$repo_dir" DESCRIPTION="$description" python3 <<'PY'
import datetime
import json
import os
import shutil
from pathlib import Path

repo_dir = Path(os.environ["REPO_DIR"])
description = os.environ["DESCRIPTION"]
binary_path = repo_dir / "target" / "release" / "dedent-paste"
asset_path = Path.home() / ".config" / "karabiner" / "assets" / "complex_modifications" / "paste-dedent-plain-text.json"
karabiner_path = Path.home() / ".config" / "karabiner" / "karabiner.json"

rule = {
    "description": description,
    "manipulators": [
        {
            "from": {
                "key_code": "v",
                "modifiers": {"mandatory": ["option"]},
            },
            "to": [{"shell_command": str(binary_path)}],
            "type": "basic",
        }
    ],
}

asset_path.parent.mkdir(parents=True, exist_ok=True)
asset_path.write_text(
    json.dumps({"title": "Dedent paste", "rules": [rule]}, ensure_ascii=False, indent=2) + "\n",
    encoding="utf-8",
)
print(f"Installed complex modification asset: {asset_path}")

if not karabiner_path.exists():
    print("Karabiner profile not found; enable the rule manually after opening Karabiner-Elements.")
    raise SystemExit

data = json.loads(karabiner_path.read_text(encoding="utf-8"))
profiles = data.get("profiles") or []
profile = next((item for item in profiles if item.get("selected")), profiles[0] if profiles else None)
if profile is None:
    print("Karabiner profile not found; enable the rule manually after opening Karabiner-Elements.")
    raise SystemExit

complex_modifications = profile.setdefault("complex_modifications", {})
rules = complex_modifications.setdefault("rules", [])
complex_modifications["rules"] = [
    item for item in rules if item.get("description") != description
]
complex_modifications["rules"].append(rule)

timestamp = datetime.datetime.now().strftime("%Y%m%d%H%M%S")
backup_path = karabiner_path.with_name(f"karabiner.json.bak-{timestamp}")
shutil.copy2(karabiner_path, backup_path)
karabiner_path.write_text(json.dumps(data, ensure_ascii=False, indent=4) + "\n", encoding="utf-8")

print(f"Backed up Karabiner profile: {backup_path}")
print(f"Updated active Karabiner profile: {karabiner_path}")
PY

if command -v karabiner_cli >/dev/null 2>&1; then
  karabiner_cli --lint-complex-modifications "$asset_path"
fi

echo "Done. Press Option+V to run dedent-paste."
