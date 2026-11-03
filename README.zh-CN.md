# alfred-jetbrains-rust

[English](README.md) | [简体中文](README.zh-CN.md)

在 Alfred 中搜索并打开 JetBrains 项目，支持跨 IDE 版本的最近项目、模糊搜索、
固定项目和 Git 工作树。

## 安装

需要 macOS、Alfred 5 Powerpack 和 JetBrains IDE。

下载 [alfred-jetbrains-rust.alfredworkflow](https://github.com/chang-open-source/alfred-jetbrains-rust/releases/latest/download/alfred-jetbrains-rust.alfredworkflow)，
双击导入 Alfred。一个安装包同时支持 Apple Silicon 和 Intel。

也可以一行命令安装，自动选择芯片架构并校验 SHA256：

```sh
curl -fsSL https://raw.githubusercontent.com/chang-open-source/alfred-jetbrains-rust/main/install.sh | bash
```

也可以使用 stable Rust 从源码构建：

```sh
git clone https://github.com/chang-open-source/alfred-jetbrains-rust.git
cd alfred-jetbrains-rust
./tool/build_jb.sh
./bin/jb install
```

在 Alfred 中输入 `jb update`，选中 **Update JetBrains workflow** 后按回车，
即可在后台更新，并通过通知提示成功或失败。输入 `jb update v0.2.0` 可安装指定版本。
匹配 `update` 的项目仍会显示在更新选项下方。

也可以重新执行上面的一行安装命令，或使用 CLI：

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

搜索文本支持以 `-` 开头。项目位于 Git 仓库子目录时，也能识别工作树和分支名称。

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
除更新入口外，Alfred 中的输入用于搜索项目，不执行终端子命令。

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

CI 和发版流程见[开发指南](docs/development.zh-CN.md)，
协作规则见 [AGENTS.md](AGENTS.md)，版本记录见 [CHANGELOG.md](CHANGELOG.md)。

[MIT](LICENSE) © 2026 MrChang0。
