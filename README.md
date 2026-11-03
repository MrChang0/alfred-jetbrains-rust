# alfred-jetbrains-rust

[English](README.md) | [简体中文](README.zh-CN.md)

Open JetBrains projects from Alfred. Supports recent projects across IDE versions,
fuzzy search, pinned projects, and Git worktrees.

## Install

Requires macOS, Alfred 5 Powerpack, and a JetBrains IDE.

Download [alfred-jetbrains-rust.alfredworkflow](https://github.com/MrChang0/alfred-jetbrains-rust/releases/latest/download/alfred-jetbrains-rust.alfredworkflow)
and double-click to import it into Alfred. One package supports Apple Silicon and Intel.

Or install with one command (automatically selects your architecture and verifies SHA256):

```sh
curl -fsSL https://raw.githubusercontent.com/MrChang0/alfred-jetbrains-rust/main/install.sh | bash
```

Or build from source with stable Rust:

```sh
git clone https://github.com/MrChang0/alfred-jetbrains-rust.git
cd alfred-jetbrains-rust
./tool/build_jb.sh
./bin/jb install
```

To update, rerun the one-command installer above, or use the CLI:

```sh
jb update           # Download the latest release and update the CLI and workflow
jb update v0.2.0    # Install a specific release
```

`jb install` installs or updates the workflow using the running binary; it does not
download a release. Updates reuse the existing workflow and preserve its keywords,
modifiers, settings, pinned projects, and forgotten projects. `jb update` also
replaces the running CLI executable, keeping terminal symlinks working. Use
`--workflow-name NAME` with `install`, `update`, or `install.sh` to select a workflow.
Release downloads are verified against SHA256SUMS before installation.

## Use

In Alfred, type `jb demo` to search projects, `idea demo` to search IntelliJ IDEA
projects, or `jb~` to list Git worktrees. Press Return to open a result.

See the [feature guide](docs/features.md) for all product keywords, search rules,
IDE detection, worktrees, pin/ignore controls, and configuration options.

| Hold while pressing Return | Action |
| --- | --- |
| `⌘` | Pin/unpin |
| `⌥` | Reveal in Finder |
| `⌃` | Copy path |
| `⇧` | Open terminal |
| `⌘⇧` | Forget project |

For terminal use, add `jb` to your `PATH`; workflow installation does not do this.
Run `jb .` to open the current directory, `jb doctor` for diagnostics, or
`jb --help` for commands. Alfred search queries are not terminal subcommands.

## Configure

Open **Alfred Preferences → Workflows → JetBrains Projects (Rust) → Configure Workflow**,
or edit `~/.config/alfred-jetbrains/config.toml`. Use `jb config` to inspect effective settings.
Alfred options override matching TOML values; ignore rules are appended.

```toml
terminal = "Terminal"
default_product = "idea"
ignore = ["~/Downloads/*"]
```

## Develop

```sh
cd rust
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
```

From the repository root, run `./tool/test_release.sh` to test version PR
generation with Node.js 20+ and npm. The pinned release tool is installed into a
temporary directory; these tests do not create GitHub PRs, tags, or releases.

GitHub Actions uses three workflows:

- `ci.yml` runs formatting checks, Clippy, Rust tests, and installer tests on
  Apple Silicon and Intel macOS for pushes to any branch and pull requests.
  A local commit triggers CI after it is pushed to GitHub.
- `release-please.yml` starts a release only when you choose **Actions →
  Release PR → Run workflow → main**. After CI passes, Conventional Commits
  such as `feat:` and `fix:` produce a separate release PR with version bumps and
  generated release notes. The bot's PR is tested explicitly, with a
  **Release PR CI** status attached to its exact commit. Ordinary pushes run CI
  without creating a release PR.
- `release.yml` builds both architectures, packages the universal workflow, and
  uploads release assets and SHA256SUMS after testing the version tag. It runs
  for pushed `v*` tags or is called directly by the release automation.

`./tool/package_workflow.sh` packages a built binary as an Alfred workflow.
Keep version bumps out of feature and fix commits. Release Please updates
Cargo.toml, Cargo.lock, CHANGELOG.md, version.txt, and its manifest in the release
PR. Use **Rebase and merge** for PRs; merge and squash commits are disabled.
Preserve the release PR's generated `chore(main): release X.Y.Z` title and
commit message;
its merge event runs CI, creates the tag and GitHub release, and calls the
asset workflow directly. Merging ordinary PRs does not start a release, and
finishing a release does not create the next version PR. This works with the
default `GITHUB_TOKEN`, whose tag events do not start another workflow.
No personal access token is required.

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
See [AGENTS.md](AGENTS.md) for contributor guidance and [CHANGELOG.md](CHANGELOG.md) for releases.

[MIT](LICENSE) © 2026 MrChang0.
