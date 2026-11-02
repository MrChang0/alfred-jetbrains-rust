#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
binary="${1:-$ROOT/bin/jb}"
output="${2:-$ROOT/dist/alfred-jetbrains-rust.alfredworkflow}"
[[ -x "$binary" ]] || { printf 'Build jb first: ./tool/build_jb.sh\n' >&2; exit 1; }
binary="$(cd "$(dirname "$binary")" && pwd)/$(basename "$binary")"
mkdir -p "$(dirname "$output")"
output="$(cd "$(dirname "$output")" && pwd)/$(basename "$output")"
workdir="$(mktemp -d "${TMPDIR:-/tmp}/alfred-jetbrains-package.XXXXXX")"
trap 'rm -rf "$workdir"' EXIT

JB_CONFIG=/dev/null "$binary" export "$workdir/workflow"
(
  cd "$workdir/workflow"
  zip -qr "$workdir/package.alfredworkflow" .
)
mv "$workdir/package.alfredworkflow" "$output"
printf 'Packaged workflow: %s\n' "$output"
