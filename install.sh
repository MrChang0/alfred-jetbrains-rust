#!/usr/bin/env bash
set -euo pipefail

fail() { printf 'Install failed: %s\n' "$*" >&2; exit 1; }

main() {
  if [[ "${1:-}" == "--help" ]]; then
    printf 'Usage: bash install.sh [latest|vX.Y.Z] [--workflow-name NAME]\nInstall or update the Alfred workflow. Existing settings are preserved.\n'
    exit 0
  fi
  version=latest
  if [[ $# -gt 0 && "$1" != --* ]]; then
    version="$1"
    shift
  fi
  install_args=(install)
  if [[ "${1:-}" == --workflow-name ]]; then
    [[ $# -eq 2 && -n "$2" ]] || fail "--workflow-name requires a name"
    install_args+=(--workflow-name "$2")
    shift 2
  fi
  [[ $# -eq 0 ]] || fail "unexpected argument: $1"
  [[ "$version" == latest || "$version" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[A-Za-z0-9.-]+)?$ ]] || fail "invalid version"
  [[ "$(uname -s)" == Darwin ]] || fail "macOS is required"
  case "$(uname -m)" in
    arm64|aarch64) arch=ARM64 ;;
    x86_64) arch=X64 ;;
    *) fail "unsupported CPU architecture" ;;
  esac
  [[ -f "$HOME/Library/Application Support/Alfred/prefs.json" ]] || fail "start Alfred once before installing"

  for command in curl tar shasum awk mktemp; do
    command -v "$command" >/dev/null || fail "missing command: $command"
  done
  base="${JB_RELEASE_REPOSITORY:-https://github.com/chang-open-source/alfred-jetbrains-rust}/releases"
  if [[ "$version" == latest ]]; then
    base="$base/latest/download"
  else
    base="$base/download/$version"
  fi
  archive="jb-$arch-apple-darwin.tar.gz"
  workdir="$(mktemp -d "${TMPDIR:-/tmp}/alfred-jetbrains-rust.XXXXXX")"
  replacement=""
  trap 'rm -rf "$workdir"; if [[ -n "$replacement" ]]; then rm -f "$replacement"; fi' EXIT

  printf 'Downloading %s (%s)...\n' "$version" "$arch"
  curl --fail --location --silent --show-error --retry 3 "$base/$archive" -o "$workdir/$archive"
  curl --fail --location --silent --show-error --retry 3 "$base/SHA256SUMS" -o "$workdir/SHA256SUMS"
  expected="$(awk -v file="$archive" '$2 == file {print $1}' "$workdir/SHA256SUMS")"
  [[ "$expected" =~ ^[a-fA-F0-9]{64}$ ]] || fail "missing or invalid SHA256 checksum"
  actual="$(shasum -a 256 "$workdir/$archive" | awk '{print $1}')"
  [[ "$actual" == "$expected" ]] || fail "SHA256 checksum mismatch"

  tar -xzf "$workdir/$archive" -C "$workdir" jb
  [[ -f "$workdir/jb" && ! -L "$workdir/jb" ]] || fail "archive does not contain a regular jb executable"
  chmod +x "$workdir/jb"
  if [[ -n "${JB_UPDATE_BINARY:-}" ]]; then
    [[ -f "$JB_UPDATE_BINARY" && ! -L "$JB_UPDATE_BINARY" ]] || fail "update target must be a regular executable"
    replacement="$(mktemp "$(dirname "$JB_UPDATE_BINARY")/.jb-update.XXXXXX")"
    cp "$workdir/jb" "$replacement"
    chmod 755 "$replacement"
  fi
  "$workdir/jb" "${install_args[@]}"
  if [[ -n "$replacement" ]]; then
    mv -f "$replacement" "$JB_UPDATE_BINARY"
    replacement=""
    printf 'Updated CLI: %s\n' "$JB_UPDATE_BINARY"
  fi
  printf 'Done. Type jb in Alfred to search projects.\n'
}

main "$@"
