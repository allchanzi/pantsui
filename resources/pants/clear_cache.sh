#!/usr/bin/env bash

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

clear_launcher_cache=false
clear_dist=false
dry_run=false
assume_yes=false

usage() {
  cat <<'EOF'
Usage: resources/pants/clear_cache.sh [options]

Clear local Pants caches for this workspace.

Options:
  --all       Also clear the Pants launcher cache (nce).
  --dist      Also delete the repo dist/ directory.
  --dry-run   Print what would be deleted without deleting it.
  --yes       Skip the confirmation prompt.
  --help      Show this help.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --all)
      clear_launcher_cache=true
      ;;
    --dry-run)
      dry_run=true
      ;;
    --yes)
      assume_yes=true
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
  shift
done

paths=(
  "${HOME}/.cache/pants/lmdb_store"
  "${HOME}/.cache/pants/named_caches"
)


if [[ "${clear_launcher_cache}" == true ]]; then
  paths+=("${HOME}/.cache/nce")
  paths+=("${HOME}/Library/Caches/nce")
fi

existing_paths=()
for path in "${paths[@]}"; do
  if [[ -e "${path}" ]]; then
    existing_paths+=("${path}")
  fi
done

if [[ "${#existing_paths[@]}" -eq 0 ]]; then
  echo "Nothing to delete."
  exit 0
fi

echo "Paths selected for deletion:"
for path in "${existing_paths[@]}"; do
  echo "  ${path}"
done

if [[ "${dry_run}" == true ]]; then
  exit 0
fi

if [[ "${assume_yes}" != true ]]; then
  printf "Delete these paths? [y/N] "
  read -r reply
  if [[ ! "${reply}" =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 1
  fi
fi

for path in "${existing_paths[@]}"; do
  rm -rf "${path}"
  echo "Deleted ${path}"
done
