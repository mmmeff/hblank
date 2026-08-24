#!/usr/bin/env bash

set -Eeuo pipefail

mode="${1:-}"

if [[ -n "$mode" && "$mode" != "--dry-run" ]]; then
  echo "usage: $0 [--dry-run]" >&2
  exit 2
fi

if [[ "$mode" != "--dry-run" ]]; then
  : "${GITHUB_REPOSITORY:?GITHUB_REPOSITORY must identify the GitHub repository}"
fi

crates=(hblank-core hblank-macros hblank hblank-cli)

crates_are_indexed() {
  local version="$1"
  local crate

  for crate in "${crates[@]}"; do
    cargo info "$crate@$version" --registry crates-io >/dev/null 2>&1 || return 1
  done
}

write_release_notes() {
  local version="$1"
  local notes_file="$2"

  awk -v version="$version" '
    $0 ~ "^#{1,2} \\[" version "\\]" { collecting = 1 }
    collecting {
      if ($0 ~ "^#{1,2} \\[" && $0 !~ "^#{1,2} \\[" version "\\]") exit
      print
    }
  ' CHANGELOG.md >"$notes_file"
}

create_release() {
  local tag="$1"
  local version="${tag#v}"

  if ! crates_are_indexed "$version"; then
    echo "Skipping $tag because its crates are not all indexed"
    return
  fi

  if [[ "$mode" == "--dry-run" ]]; then
    echo "would create GitHub release $tag"
    return
  fi

  if gh release view "$tag" --repo "$GITHUB_REPOSITORY" >/dev/null 2>&1; then
    echo "GitHub release $tag already exists; skipping"
    return
  fi

  local notes_file
  notes_file="$(mktemp)"
  write_release_notes "$version" "$notes_file"

  if [[ -s "$notes_file" ]]; then
    gh release create "$tag" \
      --repo "$GITHUB_REPOSITORY" \
      --notes-file "$notes_file"
  else
    gh release create "$tag" \
      --repo "$GITHUB_REPOSITORY" \
      --generate-notes
  fi

  rm "$notes_file"
}

tags=()
while IFS= read -r tag; do
  tags+=("$tag")
done < <(git tag --merged HEAD --list "v*" --sort=-version:refname)

limit="${#tags[@]}"
if ((limit > 20)); then
  limit=20
fi

for ((index = limit - 1; index >= 0; index--)); do
  tag="${tags[$index]}"

  if [[ "$tag" == "v0.0.0" ]]; then
    continue
  fi

  create_release "$tag"
done
