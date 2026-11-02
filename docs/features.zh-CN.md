# 功能指南

[English](features.md) | [简体中文](features.zh-CN.md) | [安装说明](../README.zh-CN.md#安装)

## 搜索与排序

工作流会合并配置目录中不同 IDE 产品、不同版本的最近项目记录。同一路径的重复记录
按最近打开时间去重，并尽可能匹配当时使用的 IDE 产品和版本。

记录来自 IDE 的最近项目 XML 文件，Fleet/Air 还会读取 `backend/**/trusted-paths.xml`。
显示名称优先使用 `.idea` 中的名称、模块或工作区信息，没有时使用目录名。

在 Alfred 中输入：

| 输入 | 功能 |
| --- | --- |
| `jb` | 列出所有最近项目 |
| `jb pngln` | 模糊匹配，例如 `pangolin` |
| `jb fory java` | 多词匹配，所有词分别匹配项目名称或路径组成部分 |
| `jb ~/Code` | 匹配用户主目录下的路径 |
| `jb 2026.2` | 匹配 IDE 版本 |
| `jb idea demo` | 仅搜索 IntelliJ IDEA 项目 |
| `idea` | 不输入搜索词，直接列出 IntelliJ IDEA 项目 |

匹配不区分大小写。无搜索词时按 `sort` 排序：`recent`（默认，最近打开）、`oldest`
（最早打开）、`name`（名称正序）、`name-desc`（名称倒序）或 `path`（路径）。
有搜索词时，匹配度优先于配置的排序方式；固定项目始终优先。

默认隐藏目录已不存在的项目。可在 Alfred 中开启 **Show stale projects**，
或运行 `jb search --show-stale` 显示。固定项目不受此项过滤影响，
但匹配忽略规则或已被遗忘的项目，即使固定也会隐藏。

## 产品关键字

各产品配置中 `keywords` 的第一项会注册为独立 Alfred 关键字。
产品 ID 可用于终端命令的 `--product` 和 `default_product` 配置。
工作流画布采用三列布局，全局搜索、文件操作和各 IDE 分组展示，
每组的项目与工作树入口连接到相邻的操作节点。

| 产品 | Alfred 关键字 | 产品 ID |
| --- | --- | --- |
| IntelliJ IDEA | `idea` | `idea` |
| PyCharm | `pycharm` | `pycharm` |
| PyCharm CE | `pycharmc` | `pycharm-ce` |
| WebStorm | `wstorm` | `webstorm` |
| PhpStorm | `pstorm` | `phpstorm` |
| Android Studio | `studio` | `android-studio` |
| CLion | `clion` | `clion` |
| CLion Nova | `nova` | `clion-nova` |
| GoLand | `goland` | `goland` |
| DataGrip | `datagrip` | `datagrip` |
| DataSpell | `dataspell` | `dataspell` |
| Rider | `rider` | `rider` |
| RubyMine | `rubymine` | `rubymine` |
| RustRover | `rustrover` | `rustrover` |
| Fleet | `fleet` | `fleet` |
| Air | `air` | `air` |
| Aqua | `aqua` | `aqua` |
| AppCode | `appcode` | `appcode` |
| Writerside | `writerside` | `writerside` |

在全局 `jb` 查询中，只有产品前缀后跟搜索词时才会限定产品。例如，`jb go` 搜索
名称或路径中的 `go`，而 `jb go demo` 仅搜索 GoLand 的 `demo` 项目。
额外配置的别名可用于全局查询前缀和 `--product`。

## 打开项目与选择 IDE

在终端执行 `jb .` 或 `jb open /path/to/project`。最近打开记录中的 IDE 启动器
可用时，优先复用该关联；新项目则根据项目文件推断 IDE，再回退到
`default_product`（默认 `idea`）。

| 项目文件 | 推断 IDE |
| --- | --- |
| `pom.xml`、`build.gradle[.kts]`、`settings.gradle[.kts]` | IntelliJ IDEA |
| `pyproject.toml`、`requirements.txt`、`setup.py`、`Pipfile` | PyCharm |
| `package.json` | WebStorm |
| `go.mod` | GoLand |
| `Cargo.toml` | RustRover |
| `composer.json` | PhpStorm |
| `Gemfile` | RubyMine |
| `CMakeLists.txt` | CLion |
| `*.sln`、`*.csproj` | Rider |

使用 `jb open . --product rustrover` 指定产品，也可通过
`--app /Applications/RustRover.app` 或 `--bin /path/to/launcher` 指定启动器。
优先使用应用包中的启动器；设置 `JB_BINARIES` 后，会用其中的目录替代 `PATH`
查找命令行启动器。没有找到启动器时，使用 macOS 默认处理方式打开路径。

Alfred 的 **Open in JetBrains** 文件操作对选中的文件夹使用相同规则，
刚克隆、尚无最近打开记录的项目也可以使用。

## Git 工作树与分支

在 Alfred 中输入 `jb~` 列出所有关联工作树，或为产品关键字加上 `~`，
例如 `idea~ demo`。终端对应命令为 `jb search --worktrees --filter demo`。

工作树通过最近项目所关联仓库的 Git 元数据发现，不会扫描无关目录。
普通项目在能读取到分支时显示当前分支，工作树结果显示工作树分支。
普通搜索默认隐藏关联工作树；设为 `hide_worktrees = false` 后也会显示。

## 快捷操作、固定与隐藏项目

在 Alfred 结果上按住修饰键并回车：

| 按键 | 操作 |
| --- | --- |
| `⌘` | 固定或取消固定，固定项目以 `★` 标记 |
| `⌥` | 在 Finder 中显示 |
| `⌃` | 复制项目路径 |
| `⇧` | 使用配置的终端应用打开目录 |
| `⌘⇧` | 遗忘项目 |

对应的终端管理命令：

```sh
jb pin /path/to/project
jb unpin /path/to/project
jb toggle-pin /path/to/project
jb pins
jb terminal /path/to/project
jb forget /path/to/project
jb forgotten
jb remember /path/to/project
jb forget --clear
```

批量隐藏项目，可在 TOML 中设置 `ignore = ["~/Downloads/*", "*-scratch"]`，
或在 Alfred 的 **Ignore globs** 中每行填写一条规则。
规则不区分大小写，同时匹配项目名称与完整路径。
`remember` 只移除遗忘标记，不会覆盖忽略规则；`forget --clear` 只清空遗忘列表。

## 配置

打开 **Alfred Preferences → Workflows → JetBrains Projects (Rust) → Configure Workflow**，
或编辑 `~/.config/alfred-jetbrains/config.toml`。文件不存在时使用默认值。
`JB_CONFIG` 可指定其他配置文件，`jb config` 查看实际生效配置。

| TOML 配置项 | 默认值 / 用途 |
| --- | --- |
| `show_stale` | `false`，是否显示目录不存在的项目 |
| `hide_worktrees` | `true`，是否在普通搜索中隐藏关联工作树 |
| `sort` | `"recent"`，还支持 `oldest`、`name`、`name-desc`、`path` |
| `default_product` | `"idea"`，回退使用的产品 ID 或别名 |
| `terminal` | 终端 CLI 默认为 `"Terminal"`；Alfred 配置页面默认是 `Ghostty` |
| `ignore` | `[]`，按项目名称与路径匹配的忽略规则 |
| `cache_seconds` | `0`，Alfred 结果缓存时长（秒），`0` 表示不缓存 |
| `settings_roots` | IDE 设置搜索根目录，默认目录见下文 |
| `app_roots` | IDE 应用搜索根目录，默认目录见下文 |
| `pins_file` | `~/.config/alfred-jetbrains/pins.json` |
| `forgotten_file` | `~/.config/alfred-jetbrains/forgotten.json` |
| `workflow_name` | `"alfred-jetbrains-rust"`，安装目录名称 |

默认设置根目录为 `~/Library/Application Support/JetBrains`、
`~/Library/Application Support/Google` 和 `~/Library/Preferences`。
默认应用根目录为 `/Applications`、`~/Applications` 和
`~/Applications/JetBrains Toolbox`。IDE 安装在其他位置时可修改这些目录。

Alfred 通过 `jb_terminal`、`jb_default_product`、`jb_show_stale`、
`jb_hide_worktrees`、`jb_sort`、`jb_settings_roots`、`jb_app_roots` 环境变量
覆盖对应的 TOML 配置，根目录列表使用冒号分隔。
`jb_ignore` 将按行填写的规则追加到 TOML 列表中。
也可通过 `search --sort name` 等命令行选项临时选择排序方式。

产品配置可覆盖 `preference_prefix`、`application_names`、`binaries` 和 `keywords`。
例如：

```toml
[products.idea]
keywords = ["ij", "idea"]
```

修改关键字后运行 `jb install` 重新生成工作流。此例中独立 Alfred 关键字变为 `ij`，
两个别名仍可用于全局查询。固定与遗忘列表分别保存在对应的 JSON 文件中。

## 诊断与打包

终端示例假定 `jb` 已加入 `PATH`，否则使用可执行文件的完整路径。
`jb search` 输出供 Alfred 使用的 JSON，诊断日志写入 stderr。

```sh
jb doctor
jb --debug search --filter demo
jb search --product idea --filter demo --sort name
jb config
jb install
jb export /path/to/workflow
jb --version
jb --help
jb search --help
```

`doctor` 显示发现的设置目录、IDE 应用、项目与工作树数量、隐藏项目相关配置及
工作流安装状态。也可通过 `JB_DEBUG=1` 或 Alfred 的 `alfred_debug` 环境变量启用日志。

`install` 安装或更新工作流，也能更新通过原生安装包导入的工作流。
`--workflow-name NAME` 指定安装目录，`--retain` 保留该目录现有的二进制。
`export DIRECTORY` 仅导出可移植的工作流目录，不执行安装。
在仓库根目录运行 `./tool/package_workflow.sh`，可将已构建的二进制打包为 `.alfredworkflow`。
