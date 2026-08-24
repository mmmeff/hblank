#!/usr/bin/env bash

set -Eeuo pipefail

version="${1:-}"
mode="${2:-}"

if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+([+-][0-9A-Za-z.-]+)?$ ]]; then
  echo "usage: $0 <semver> [--dry-run]" >&2
  exit 2
fi
if [[ -n "$mode" && "$mode" != "--dry-run" ]]; then
  echo "unknown option: $mode" >&2
  exit 2
fi

crates=(hblank-macros hblank hblank-cli)
node scripts/release-version.mjs check "$version"

if [[ "$mode" == "--dry-run" ]]; then
  for crate in "${crates[@]}"; do
    cargo package --allow-dirty --list -p "$crate" >/dev/null
    echo "would publish $crate@$version"
  done
  exit 0
fi

: "${CARGO_REGISTRY_TOKEN:?CARGO_REGISTRY_TOKEN must contain a crates.io token}"

crate_is_indexed() {
  local crate="$1"
  cargo info "$crate@$version" --registry crates-io >/dev/null 2>&1
}

wait_until_indexed() {
  local crate="$1"
  local attempt

  for attempt in {1..60}; do
    if crate_is_indexed "$crate"; then
      echo "$crate@$version is available from the crates.io index"
      return 0
    fi
    sleep 5
  done

  echo "timed out waiting for $crate@$version to reach the crates.io index" >&2
  return 1
}

publish_crate() {
  local crate="$1"

  if crate_is_indexed "$crate"; then
    echo "$crate@$version is already published; skipping"
    return 0
  fi

  cargo publish --locked -p "$crate"
  wait_until_indexed "$crate"
}

for crate in "${crates[@]}"; do
  publish_crate "$crate"
done
