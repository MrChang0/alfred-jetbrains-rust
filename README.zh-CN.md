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

CI 在 Apple Silicon 和 Intel macOS 上执行检查与 release 构建。
`./tool/package_workflow.sh` 可将已构建的二进制打包为工作流；
推送版本标签后会自动发布通过测试的二进制和通用工作流安装包。
协作规则见 [AGENTS.md](AGENTS.md)，版本记录见 [CHANGELOG.md](CHANGELOG.md)。

[MIT](LICENSE) © 2026 MrChang0。
