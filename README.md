# alfred-jetbrains-rust

[English](README.md) | [简体中文](README.zh-CN.md)

Open JetBrains projects from Alfred. Supports recent projects across IDE versions,
fuzzy search, pinned projects, and Git worktrees.

## Install

Requires macOS, Alfred 5 Powerpack, and a JetBrains IDE.

Download [alfred-jetbrains-rust.alfredworkflow](https://github.com/chang-open-source/alfred-jetbrains-rust/releases/latest/download/alfred-jetbrains-rust.alfredworkflow)
and double-click to import it into Alfred. One package supports Apple Silicon and Intel.

Or install with one command (automatically selects your architecture and verifies SHA256):

```sh
curl -fsSL https://raw.githubusercontent.com/chang-open-source/alfred-jetbrains-rust/main/install.sh | bash
```

Or build from source with stable Rust:

```sh
git clone https://github.com/chang-open-source/alfred-jetbrains-rust.git
cd alfred-jetbrains-rust
./tool/build_jb.sh
./bin/jb install
```

To update in Alfred, type `jb update`, select **Update JetBrains workflow**, and
press Return. The update runs in the background, with notifications for completion
or failure. Use `jb update v0.2.0` to install a specific release. Projects matching
`update` remain available below the update action.

You can also rerun the one-command installer above, or use the CLI:

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
`jb --help` for commands. Apart from the update action, Alfred search queries
search projects rather than execute terminal subcommands.

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

See the [CI and release guide](docs/development.md) for automation details,
[AGENTS.md](AGENTS.md) for contributor guidance, and [CHANGELOG.md](CHANGELOG.md) for releases.

[MIT](LICENSE) © 2026 MrChang0.
