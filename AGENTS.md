# Contributor Guide

Applies to the entire repository. Communicate in Chinese by default; keep code,
CLI help, and this file in English.

## Project

- Rust crate: `rust/`; executable: `jb`; integration tests: `rust/tests/cli.rs`.
- Build: `./tool/build_jb.sh` produces `bin/jb`.
- Configuration: `~/.config/alfred-jetbrains/`, overridable with `JB_CONFIG`.
- Workflow ID: `com.mrchang0.alfred-jetbrains-rust`.

## Validate

Run from the repository root:

```sh
cargo fmt --manifest-path rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rust/Cargo.toml --locked --all-targets --all-features -- -D warnings
cargo test --manifest-path rust/Cargo.toml --locked --all-targets --all-features
```

Add regression tests for behavior changes; documentation-only edits need no new
tests. Use temporary fixtures and mock IDEs, never real user settings or workflows.
Do not install into Alfred unless explicitly requested.

## Preserve

- Valid Alfred JSON on search stdout; diagnostics on stderr.
- Existing keywords, modifiers, configuration, and stored user state.
- Recent-IDE selection and pin priority; ignore/forget rules override pins.
- Git metadata discovery without full-disk scans or per-result Git processes.
- Safe paths via `Path`/`OsString` and `Command::arg`, including non-UTF-8 paths.

Keep changes focused, preserve unrelated user edits, and update both READMEs when
usage changes. Commit `Cargo.lock`, not binaries, build output, or secrets.
Preserve MIT notices. Read workflow version and repository URL from Cargo metadata.
For requested releases, update the package version in Cargo.toml/Cargo.lock and
CHANGELOG.md, then tag `vX.Y.Z` after CI passes. Do not bump versions, push, tag,
or publish releases unless requested.
