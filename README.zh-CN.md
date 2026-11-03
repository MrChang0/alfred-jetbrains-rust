# alfred-jetbrains-rust

[English](README.md) | [简体中文](README.zh-CN.md)

在 Alfred 中搜索并打开 JetBrains 项目，支持跨 IDE 版本的最近项目、模糊搜索、
固定项目和 Git 工作树。

## 安装

需要 macOS、Alfred 5 Powerpack 和 JetBrains IDE。

下载 [alfred-jetbrains-rust.alfredworkflow](https://github.com/MrChang0/alfred-jetbrains-rust/releases/latest/download/alfred-jetbrains-rust.alfredworkflow)，
双击导入 Alfred。一个安装包同时支持 Apple Silicon 和 Intel。

也可以一行命令安装，自动选择芯片架构并校验 SHA256：

```sh
curl -fsSL https://raw.githubusercontent.com/MrChang0/alfred-jetbrains-rust/main/install.sh | bash
```

也可以使用 stable Rust 从源码构建：

```sh
git clone https://github.com/MrChang0/alfred-jetbrains-rust.git
cd alfred-jetbrains-rust
./tool/build_jb.sh
./bin/jb install
```

更新时，重新执行上面的一行安装命令，或使用 CLI：

```sh
jb update           # 下载最新版并更新 CLI 和工作流
jb update v0.2.0    # 安装指定版本
```

`jb install` 使用当前运行的二进制安装或更新工作流，不会下载版本。
更新会复用已有工作流，保留关键字、修饰键、设置、固定和遗忘项目。
`jb update` 还会替换当前 CLI 可执行文件，终端中的符号链接仍然有效。
`install`、`update` 和 `install.sh` 均支持通过 `--workflow-name NAME` 指定工作流。
下载版本均在安装前通过 SHA256SUMS 校验。

## 使用

在 Alfred 中输入 `jb demo` 搜索项目，`idea demo` 仅搜索 IntelliJ IDEA 项目，
`jb~` 列出 Git 工作树。选中结果后按回车打开。

完整产品关键字、搜索规则、IDE 识别、工作树、固定与忽略及配置项见
[功能指南](docs/features.zh-CN.md)。

| 按住按键并回车 | 操作 |
| --- | --- |
| `⌘` | 固定或取消固定 |
| `⌥` | 在 Finder 中显示 |
| `⌃` | 复制路径 |
| `⇧` | 打开终端 |
| `⌘⇧` | 遗忘项目 |

如需在终端使用，请自行将 `jb` 加入 `PATH`，安装工作流不会自动配置。
`jb .` 打开当前目录，`jb doctor` 查看诊断，`jb --help` 查看命令。
Alfred 中的搜索输入不是终端子命令。

## 配置

打开 **Alfred Preferences → Workflows → JetBrains Projects (Rust) → Configure Workflow**，
或编辑 `~/.config/alfred-jetbrains/config.toml`。用 `jb config` 查看生效配置。
Alfred 选项覆盖对应的 TOML 值，忽略规则则追加到 TOML 规则中。

```toml
terminal = "Terminal"
default_product = "idea"
ignore = ["~/Downloads/*"]
```

## 开发

```sh
cd rust
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-targets --all-features
```

在仓库根目录运行 `./tool/test_release.sh` 可验证版本 PR 的生成，需要 Node.js 20+
和 npm。测试工具安装在临时目录中，不会在 GitHub 创建 PR、标签或发布版本。

GitHub Actions 分为三个工作流：

- `ci.yml`：向任意分支推送代码、创建或更新 PR 时，在 Apple Silicon 和
  Intel macOS 上执行格式检查、Clippy、Rust 测试和安装脚本测试。
  本地 commit 需要推送到 GitHub 后才会触发 CI。
- `release-please.yml`：准备发版时，手动选择 **Actions → Release PR →
  Run workflow → main**。CI 通过后，根据 `feat:`、`fix:` 等 Conventional Commits
  创建或更新独立的版本 PR，更新版本号和发布说明。普通 push 只运行 CI，不创建版本 PR。
  机器人创建的 PR 会显式运行测试，并在对应提交上显示 **Release PR CI** 状态。
- `release.yml`：测试版本标签后，构建两种架构、打包通用工作流并上传安装包和
  SHA256SUMS。支持推送 `v*` 标签触发，也可由版本自动化直接调用。

`./tool/package_workflow.sh` 可将已构建的二进制打包为工作流。
功能和修复提交不再手动升级版本。Release Please 会在独立版本 PR 中统一更新
Cargo.toml、Cargo.lock、CHANGELOG.md、version.txt 和发布版本清单。
PR 统一使用 **Rebase and merge**，仓库已禁用 merge commit 和 squash merge。
合并版本 PR 时，保留自动生成的 `chore(main): release X.Y.Z` 标题与提交说明；
合并事件会运行 CI，通过后自动创建标签和 GitHub Release，并直接调用安装包构建流程。
合并普通 PR 不会触发发版；完成发版后也不会自动创建下一个版本 PR。
整个流程使用默认 `GITHUB_TOKEN`，无需个人访问令牌，也不依赖机器人标签事件触发新任务。

在 **Run workflow** 中选择 `release_kind`。以当前版本 `0.2.0` 为例：

| 选项 | 生成版本 |
| --- | --- |
| `auto` | 根据 Conventional Commits 推算 |
| `patch` | `0.2.1`，补丁版本 |
| `minor` | `0.3.0`，次版本 |
| `major` | `1.0.0`，主版本 |
| `prepatch` | `0.2.1-beta.1` |
| `preminor` | `0.3.0-beta.1` |
| `premajor` | `1.0.0-beta.1` |
| `prerelease` | 递增已有预发布；从正式版开始则生成下一个补丁的预发布 |
| `custom` | 使用 `version` 输入的指定版本，例如 `1.0.0-rc.1` |

`prerelease_id` 为预发布选择 `alpha`、`beta` 或 `rc` 通道。
例如 `prerelease` 配合 `beta`，会把 `0.3.0-beta.1` 升为 `0.3.0-beta.2`；
选择 `patch` 可将 `0.3.0-beta.2` 转为正式版 `0.3.0`。仅 `custom` 使用 `version` 输入。
明确指定升级类型时，即使没有新代码变更，也能创建版本 PR。
新版本必须高于当前版本，不接受降级或构建元数据。
预发布会标记为 GitHub Pre-release，可用 `jb update v0.3.0-beta.1` 指定安装；
不指定版本的 `jb update` 仍使用最新正式版。

仓库需要一次性启用 **Settings → Actions → General → Workflow permissions →
Allow GitHub Actions to create and approve pull requests**。
`version.txt` 和 `.release-please-manifest.json` 由自动化维护，记录已发布版本；
CLI 和工作流版本仍读取 Cargo 元数据。
协作规则见 [AGENTS.md](AGENTS.md)，版本记录见 [CHANGELOG.md](CHANGELOG.md)。

[MIT](LICENSE) © 2026 MrChang0。
