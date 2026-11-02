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

CI runs checks and release builds on Apple Silicon and Intel macOS.
`./tool/package_workflow.sh` packages a built binary as an Alfred workflow;
version tags publish the tested binaries and universal workflow automatically.
See [AGENTS.md](AGENTS.md) for contributor guidance and [CHANGELOG.md](CHANGELOG.md) for releases.

[MIT](LICENSE) © 2026 MrChang0.
