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
./tool/test_release.sh
```

Release automation regression tests require Node.js 20+ and npm; they install
the pinned release tool in a temporary directory and make no GitHub changes.

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
Keep package-version bumps out of feature and fix commits. Start a release
manually with Actions > Release PR > Run workflow on main; ordinary pushes run
CI only. Select the release kind and optional preview channel or exact version
in the dispatch form. Release Please owns version changes in Cargo.toml/Cargo.lock,
CHANGELOG.md, version.txt, and its manifest through a separate release PR.
Use Rebase and merge for PRs. Preserve the release PR's generated
`chore(main): release X.Y.Z` title and commit message.
After CI passes, automation creates `vX.Y.Z`
and publishes release assets. Do not bump versions, push, tag, or publish
releases unless requested.
