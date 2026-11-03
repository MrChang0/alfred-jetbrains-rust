# Contributor Guide

## Changes and validation

- Keep changes focused and preserve unrelated user edits.
- Update both READMEs when usage changes; preserve MIT notices.
- Run formatting, Clippy, Rust tests, and relevant script tests before submitting.
  Add regression tests for behavior changes; documentation-only edits need no new tests.
- Use temporary fixtures and mock IDEs, never real user settings or workflows.
- Commit `Cargo.lock`, not binaries, build output, or secrets.
- Do not install into Alfred, push, tag, or publish releases unless requested.
- Leave version bumps to release automation; read version and repository URL from Cargo metadata.

## Preserve

- Valid Alfred JSON on search stdout; diagnostics on stderr.
- Existing keywords, modifiers, configuration, and stored user state.
- Workflow bundle ID and compatibility with existing installations.
- Recent-IDE selection and pin priority; ignore/forget rules override pins.
- Git metadata discovery without full-disk scans or per-result Git processes.
- Safe paths via `Path`/`OsString` and `Command::arg`, including non-UTF-8 paths.
