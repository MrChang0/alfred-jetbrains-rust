# CI 与发布

[English](development.md) | [简体中文](development.zh-CN.md) | [安装说明](../README.zh-CN.md)

GitHub Actions 分为三个工作流：

- `ci.yml`：向任意分支推送代码、创建或更新 PR 时，在 Apple Silicon 和
  Intel macOS 上执行格式检查、Clippy、Rust 测试和安装脚本测试。
  本地 commit 需要推送到 GitHub 后才会触发 CI。
- `release-please.yml`：准备发版时，手动选择 **Actions → Release PR →
  Run workflow → main**。CI 通过后，根据 `feat:`、`fix:` 等 Conventional Commits
  创建或更新独立的版本 PR，更新版本号和发布说明。普通 push 只运行 CI，不创建版本 PR。
  机器人创建的 PR 会显式运行测试，并在对应提交上显示 **Release PR CI** 状态。
  测试通过后自动 rebase 合并版本 PR，重新测试合并后的提交，再继续发布。
- `release.yml`：测试版本标签后，构建两种架构、打包通用工作流并上传安装包和
  SHA256SUMS。支持推送 `v*` 标签触发，也可由版本自动化直接调用。

`./tool/package_workflow.sh` 可将已构建的二进制打包为工作流。
功能和修复提交不再手动升级版本。Release Please 会在独立版本 PR 中统一更新
Cargo.toml、Cargo.lock、CHANGELOG.md、version.txt 和发布版本清单。
PR 统一使用 **Rebase and merge**，仓库已禁用 merge commit 和 squash merge。
版本 PR 无需手动合并，保留自动生成的 `chore(main): release X.Y.Z` 标题与提交说明即可。
流程仅自动合并测试通过的原始 PR 提交，且变更必须限定为版本文件和 CHANGELOG.md。
rebase 后会重新测试，再自动创建标签和 GitHub Release，调用安装包构建流程。
测试失败或 PR 提交发生变化时会停止发布。普通 PR 仍手动合并，不会触发发版；
完成发版后也不会自动创建下一个版本 PR。
整个流程使用默认 `GITHUB_TOKEN`，无需个人访问令牌。机器人合并及标签事件不会触发新任务，
因此同一次运行会显式继续执行测试和发布。

在 **Run workflow** 中选择 `release_kind`。以 `0.2.0` 为起点举例：

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
协作规则见 [AGENTS.md](../AGENTS.md)，版本记录见 [CHANGELOG.md](../CHANGELOG.md)。
