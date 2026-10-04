#!/usr/bin/env bash

set -euo pipefail

CRATE_NAME="tempoch-py"
DRY_RUN=false

usage() {
  echo "Usage: $0 [--dry-run]" >&2
}

if [[ "${1:-}" == "--dry-run" ]]; then
  DRY_RUN=true
  shift
fi

if [[ "$#" -ne 0 ]]; then
  usage
  exit 2
fi

read_version() {
  local section="$1"
  local file="$2"

  awk -v target="$section" '
    $0 == "[" target "]" {
      in_section = 1
      next
    }
    in_section && /^\[/ {
      exit
    }
    in_section && $1 == "version" && $2 == "=" {
      gsub(/"/, "", $3)
      print $3
      exit
    }
  ' "$file"
}

cargo_version="$(read_version package Cargo.toml)"
python_version="$(read_version project pyproject.toml)"

if [[ -z "$cargo_version" || -z "$python_version" ]]; then
  echo "error: could not read Cargo/Python package versions" >&2
  exit 1
fi

if [[ "$cargo_version" != "$python_version" ]]; then
  echo "error: version mismatch: Cargo.toml=$cargo_version pyproject.toml=$python_version" >&2
  exit 1
fi

expected_tag="v$cargo_version"
release_tag="${RELEASE_TAG:-${GITHUB_REF_NAME:-}}"

if [[ -z "$release_tag" ]]; then
  release_tag="$(git describe --tags --exact-match HEAD 2>/dev/null || true)"
fi

if [[ -z "$release_tag" ]]; then
  if [[ "$DRY_RUN" == true ]]; then
    release_tag="$expected_tag"
    echo "No exact Git tag detected; validating dry-run against $expected_tag"
  else
    echo "error: publication requires an exact release tag" >&2
    exit 1
  fi
fi

if [[ "$release_tag" != "$expected_tag" ]]; then
  echo "error: tag/version mismatch: tag=$release_tag Cargo.toml=$cargo_version" >&2
  exit 1
fi

registry_url="https://crates.io/api/v1/crates/$CRATE_NAME/$cargo_version"
registry_response="$(mktemp)"
trap 'rm -f "$registry_response"' EXIT

http_code="$(
  curl --silent --show-error \
    --output "$registry_response" \
    --write-out "%{http_code}" \
    "$registry_url"
)" || {
  echo "error: failed to query crates.io" >&2
  exit 1
}

case "$http_code" in
  200)
    already_published=true
    ;;
  404)
    already_published=false
    ;;
  *)
    echo "error: crates.io returned HTTP $http_code while checking $CRATE_NAME $cargo_version" >&2
    cat "$registry_response" >&2
    exit 1
    ;;
esac

if [[ "$already_published" == true && "$DRY_RUN" == false ]]; then
  echo "$CRATE_NAME $cargo_version is already published; nothing to do."
  exit 0
fi

cargo package --locked

if [[ "$already_published" == false ]]; then
  cargo publish --dry-run --locked
else
  echo "$CRATE_NAME $cargo_version already exists on crates.io; skipping duplicate publish dry-run."
fi

if [[ "$DRY_RUN" == true ]]; then
  echo "Release validation succeeded for $expected_tag."
  exit 0
fi

cargo publish --locked
