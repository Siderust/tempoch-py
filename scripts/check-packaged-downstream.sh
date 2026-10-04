#!/usr/bin/env bash

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

CRATE_NAME="tempoch-py"

read_version() {
  awk '
    $0 == "[package]" {
      in_package = 1
      next
    }
    in_package && /^\[/ {
      exit
    }
    in_package && $1 == "version" && $2 == "=" {
      gsub(/"/, "", $3)
      print $3
      exit
    }
  ' Cargo.toml
}

crate_version="$(read_version)"
if [[ -z "$crate_version" ]]; then
  echo "error: could not read package version from Cargo.toml" >&2
  exit 1
fi

cargo package --locked

crate_archive="target/package/$CRATE_NAME-$crate_version.crate"
if [[ ! -f "$crate_archive" ]]; then
  echo "error: expected packaged crate not found: $crate_archive" >&2
  exit 1
fi

workdir="$(mktemp -d)"
trap 'rm -rf "$workdir"' EXIT

tar -xzf "$crate_archive" -C "$workdir"
package_dir="$workdir/$CRATE_NAME-$crate_version"
consumer_dir="$workdir/downstream-extension"
cp -R tests/fixtures/downstream-extension "$consumer_dir"
rm -f "$consumer_dir/Cargo.lock"

python3 - "$consumer_dir/Cargo.toml" "$crate_version" <<'PY'
from pathlib import Path
import re
import sys

manifest = Path(sys.argv[1])
version = sys.argv[2]
text = manifest.read_text()
text, count = re.subn(
    r'^tempoch-py\s*=.*$',
    f'tempoch-py = "={version}"',
    text,
    count=1,
    flags=re.MULTILINE,
)
if count != 1:
    raise SystemExit("error: could not replace tempoch-py dependency in downstream fixture")
manifest.write_text(text)
PY

if grep -Eq '^tempoch-py\s*=.*path\s*=' "$consumer_dir/Cargo.toml"; then
  echo "error: packaged downstream validation still contains a path dependency" >&2
  exit 1
fi

cargo_home="$workdir/cargo-home"
mkdir -p "$cargo_home"
cat > "$cargo_home/config.toml" <<EOF
[patch.crates-io]
tempoch-py = { path = "$package_dir" }
EOF

echo "Validating downstream consumer against packaged $CRATE_NAME $crate_version"
CARGO_HOME="$cargo_home" cargo check --manifest-path "$consumer_dir/Cargo.toml"
