#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
test_dir="$(mktemp -d "${TMPDIR:-/tmp}/jb-release-tests.XXXXXX")"
trap 'rm -rf "$test_dir"' EXIT

# Match the library installed by the Release PR workflow.
npm_config_cache="$test_dir/cache" npm install --prefix "$test_dir" \
  --ignore-scripts --no-audit --no-fund --no-package-lock release-please@17.3.0
RELEASE_PLEASE_MODULE="$test_dir/node_modules/release-please" \
  node --test "$ROOT/tool/test_release.cjs"
