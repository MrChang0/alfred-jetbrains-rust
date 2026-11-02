# 项目协作指南

本文件适用于整个仓库。项目是 macOS 上用于 Alfred 5 的 JetBrains 项目启动器，
以 Rust 实现，CLI 可执行文件名为 `jb`。默认用中文沟通；代码标识符、CLI 帮助与
现有英文文档保持一致。

## 仓库结构

- `rust/Cargo.toml`、`rust/Cargo.lock`：唯一的 Rust crate 和锁定依赖。
- `rust/src/main.rs`：CLI 命令、参数归一化及错误输出。
- `rust/src/config.rs`、`product.rs`：TOML/Alfred 环境变量配置和产品定义。
- `rust/src/discovery.rs`、`projects.rs`、`xml.rs`：设置、应用与最近项目发现。
- `rust/src/search.rs`、`alfred.rs`：过滤、排序与 Alfred Script Filter JSON。
- `rust/src/worktree.rs`：直接读取 Git 元数据以发现工作树及分支。
- `rust/src/open.rs`、`install.rs`：打开 IDE/终端及生成 Alfred 工作流。
- `rust/src/pins.rs`、`forgotten.rs`：用户持久化状态。
- `rust/tests/cli.rs`：通过临时目录和模拟应用运行真实 CLI 的集成测试。
- `rust/assets/icon.png`：安装时嵌入的工作流图标。
- `tool/build_jb.sh`：从任意工作目录构建并复制 release 二进制到 `bin/jb`。
- `.github/workflows/ci.yml`：Apple Silicon 和 Intel macOS 的自动检查与构建。

## 构建和验证

需要 stable Rust（edition 2024），由根目录的 `rust-toolchain.toml` 声明。
在仓库根目录执行：

```sh
cargo fmt --manifest-path rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rust/Cargo.toml --locked --all-targets --all-features -- -D warnings
cargo test --manifest-path rust/Cargo.toml --locked --all-targets --all-features
./tool/build_jb.sh
./bin/jb --version
```

修改行为时补充有价值的回归测试；文档变更不要求新增测试。涉及 CLI、Alfred JSON、
工作流生成或构建脚本时，应执行相关检查并说明结果。不要通过屏蔽警告或跳过失败
测试来使 CI 通过。提交 `Cargo.lock`，不要提交 `rust/target/` 或 `bin/jb`。

## 行为约束

- `jb search` 的 stdout 必须是合法的 Alfred JSON；诊断信息写入 stderr。
  搜索失败时保留可展示的 Alfred 错误项，其他 CLI 命令保持失败退出码。
- 保持 `jb` 与产品关键字、`~` 工作树关键字和现有修饰键兼容。
- 同一项目按最近打开记录选择 IDE；固定项目优先；忽略/遗忘规则高于固定状态。
- 通过 `.git` 元数据发现工作树，避免全盘扫描或为每个结果启动 Git 子进程。
- 路径可能含空格、Unicode 或非 UTF-8 字节。优先使用 `Path`/`PathBuf`/`OsString`
  和 `Command::arg`，不要通过拼接 shell 命令传递用户路径。
- 配置仍位于 `~/.config/alfred-jetbrains/`；`JB_CONFIG` 可覆盖配置路径。
  保持 Alfred `jb_*` 环境变量覆盖逻辑，以及现有 pins/forgotten 文件兼容性。
- 工作流 bundle ID 是 `com.mrchang0.alfred-jetbrains-rust`；其版本和仓库地址
  从 Cargo 元数据读取，避免重复硬编码。

## 测试隔离

测试必须使用临时目录、配置和模拟 IDE；在测试子进程中设置所需环境。
不要修改当前进程的全局环境以免并行测试互相干扰。不要读取或改写开发者真实的
Alfred 工作流、IDE 最近项目、配置、固定或遗忘列表。`jb install` 会改写 Alfred
首选项中的工作流，验证安装时必须使用测试夹具，除非用户明确要求真实安装。

## 版本和文档

本仓库从 `0.1.0` 开始独立版本序列，迁入来源是旧仓库 `0.3.1` 的 Rust 实现。
发布时同步 `rust/Cargo.toml` 和 `rust/Cargo.lock` 中本包版本，更新 `CHANGELOG.md`，
通过 CI 后创建同版本 `vX.Y.Z` 标签。CLI 与工作流从 `CARGO_PKG_VERSION` 取版本。
不要在未获请求时自动升级版本、推送、打标签或发布 Release。

增加命令、配置或关键字时同步更新根 `README.md`、`rust/README.md` 与相关测试。
许可证为 MIT；保留已有版权和许可声明。不要提交 token、个人配置、IDE 缓存或
真实项目历史。只提交与当前任务相关的修改，保留用户已有的未提交工作。
