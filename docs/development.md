# CI and releases

[English](development.md) | [简体中文](development.zh-CN.md) | [README](../README.md)

GitHub Actions uses three workflows:

- `ci.yml` runs formatting checks, Clippy, Rust tests, and installer tests on
  Apple Silicon and Intel macOS for pushes to any branch and pull requests.
  A local commit triggers CI after it is pushed to GitHub.
- `release-please.yml` starts a release only when you choose **Actions →
  Release PR → Run workflow → main**. After CI passes, Conventional Commits
  such as `feat:` and `fix:` produce a separate release PR with version bumps and
  generated release notes. The bot's PR is tested explicitly, with a
  **Release PR CI** status attached to its exact commit. After that check passes,
  the workflow automatically rebases the version PR and tests the resulting
  commit before publishing. Ordinary pushes run CI without creating a release PR.
- `release.yml` builds both architectures, packages the universal workflow, and
  uploads release assets and SHA256SUMS after testing the version tag. It runs
  for pushed `v*` tags or is called directly by the release automation.

`./tool/package_workflow.sh` packages a built binary as an Alfred workflow.
Keep version bumps out of feature and fix commits. Release Please updates
Cargo.toml, Cargo.lock, CHANGELOG.md, version.txt, and its manifest in the release
PR. Use **Rebase and merge** for PRs; merge and squash commits are disabled.
Release PRs are merged automatically; preserve their generated
`chore(main): release X.Y.Z` title and commit message. The workflow only merges
the tested PR head and only if its changes are limited to release version files
and CHANGELOG.md. It tests the rebased commit, creates the tag and GitHub
release, and calls the asset workflow directly. Failures or a changed PR head
stop the release. Other PRs are still merged manually and do not start a release;
finishing a release does not create the next version PR. This works with the
default `GITHUB_TOKEN`: merge and tag events from this token do not start another
workflow, so the same run explicitly continues through testing and publishing.
No personal access token is required.

Protect `main` with the required checks `Check (macos-latest)`,
`Check (macos-15-intel)`, and `Release automation tests`. The release workflow
mirrors its complete test result under these names on the exact version PR head,
so the same protection applies to ordinary PRs and release PRs. No approving
review is required for this single-maintainer workflow.

In **Run workflow**, select `release_kind`. Starting from `0.2.0`:

| Selection | Result |
| --- | --- |
| `auto` | Derive the next version from Conventional Commits |
| `patch` | `0.2.1` |
| `minor` | `0.3.0` |
| `major` | `1.0.0` |
| `prepatch` | `0.2.1-beta.1` |
| `preminor` | `0.3.0-beta.1` |
| `premajor` | `1.0.0-beta.1` |
| `prerelease` | Increment an existing preview; from stable, start the next patch preview |
| `custom` | Use the exact `version` input, e.g. `1.0.0-rc.1` |

`prerelease_id` selects `alpha`, `beta`, or `rc` for the pre* choices.
For example, `prerelease` with `beta` changes `0.3.0-beta.1` to `0.3.0-beta.2`;
select `patch` to promote `0.3.0-beta.2` to stable `0.3.0`. Only `custom` accepts
the `version` field. Explicit selections can create a version PR without new
code changes. Versions must increase; downgrades and build metadata are rejected.
Preview tags are published as GitHub pre-releases. Use `jb update v0.3.0-beta.1`
to install one explicitly; `jb update` continues to use the latest stable release.

Enable **Settings → Actions → General → Workflow permissions → Allow GitHub
Actions to create and approve pull requests** once for the repository.
`version.txt` and `.release-please-manifest.json` track the released version for
automation; the CLI and workflow version still come from Cargo metadata.
See [AGENTS.md](../AGENTS.md) for contributor guidance and [CHANGELOG.md](../CHANGELOG.md) for releases.
