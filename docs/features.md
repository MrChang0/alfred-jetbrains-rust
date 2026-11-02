# Feature Guide

[English](features.md) | [简体中文](features.zh-CN.md) | [Install](../README.md#install)

## Search and sorting

The workflow merges recent-project records across configured IDE products and
versions. Duplicate paths use the most recent project-open record. When possible,
the installed IDE is matched to that product and version.

Records come from the IDE's recent-project XML files; Fleet/Air also use
`backend/**/trusted-paths.xml`. Display names use available `.idea` name, module,
or workspace metadata before falling back to the directory name.

Enter these queries in Alfred:

| Query | Result |
| --- | --- |
| `jb` | All recent projects |
| `jb pngln` | Fuzzy match, such as `pangolin` |
| `jb fory java` | Match all words across project names and path components |
| `jb ~/Code` | Match a home-relative path |
| `jb 2026.2` | Match an IDE version |
| `jb idea demo` | Search IntelliJ IDEA projects |
| `idea` | List IntelliJ IDEA projects without a search term |

Matching is case-insensitive. Without a query, `sort` controls the order:
`recent` (default), `oldest`, `name`, `name-desc`, or `path`. With a query,
relevance comes before the configured sort order. Pinned projects remain first.

Missing directories are hidden by default. Enable **Show stale projects** or use
`jb search --show-stale` to include them. Pinned projects survive this filter;
ignored or forgotten projects remain hidden even when pinned.

## Product keywords

Each product's first configured keyword is registered as an Alfred keyword.
Product IDs also work with CLI `--product` and the `default_product` setting.
The workflow canvas uses a three-column layout, with global search, file actions,
and each IDE in separate groups. Each search/worktree pair connects to a nearby action.

| Product | Alfred keyword | Product ID |
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

A product prefix inside the global `jb` query is consumed only when followed by
a search term. For example, `jb go` searches for `go`; `jb go demo` scopes `demo`
to GoLand. Additional configured aliases work as query prefixes and with `--product`.

## Open projects and choose an IDE

In a terminal, run `jb .` or `jb open /path/to/project`. The CLI reuses a recent
IDE association when its launcher is available. For new projects, it infers an IDE
from project files, then falls back to `default_product` (default `idea`).

| Project marker | Inferred IDE |
| --- | --- |
| `pom.xml`, `build.gradle[.kts]`, `settings.gradle[.kts]` | IntelliJ IDEA |
| `pyproject.toml`, `requirements.txt`, `setup.py`, `Pipfile` | PyCharm |
| `package.json` | WebStorm |
| `go.mod` | GoLand |
| `Cargo.toml` | RustRover |
| `composer.json` | PhpStorm |
| `Gemfile` | RubyMine |
| `CMakeLists.txt` | CLion |
| `*.sln`, `*.csproj` | Rider |

Override the selection with `jb open . --product rustrover`, or specify a launcher
with `--app /Applications/RustRover.app` or `--bin /path/to/launcher`.
The app bundle's launcher is preferred; `JB_BINARIES` supplies alternative launcher
directories instead of `PATH`. If no launcher is found, macOS opens the path using
its default handler.

Alfred's **Open in JetBrains** file action applies the same selection to a chosen
folder, including a fresh clone that has no recent-project record.

## Git worktrees and branches

Enter `jb~` for all linked worktrees, or a product keyword with `~`, such as
`idea~ demo`. In a terminal, use `jb search --worktrees --filter demo`.

Worktrees are discovered from recent-project repositories through Git metadata;
unrelated directories are not scanned. Ordinary results show the current branch
when available, and worktree results show their worktree branch. Normal search
hides linked worktrees unless `hide_worktrees = false`.

## Quick actions, pins, and hidden projects

Hold a modifier and press Return on an Alfred result:

| Modifier | Action |
| --- | --- |
| `⌘` | Pin/unpin; pinned projects have a `★` prefix |
| `⌥` | Reveal in Finder |
| `⌃` | Copy the project path |
| `⇧` | Open the directory with the configured terminal application |
| `⌘⇧` | Forget the project |

Equivalent terminal controls:

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

For broad exclusions, set `ignore = ["~/Downloads/*", "*-scratch"]` in TOML or
add one glob per line in Alfred's **Ignore globs** field. Rules match project names
and full paths case-insensitively. `remember` removes a forgotten entry but does
not override ignore rules; `forget --clear` clears only the forgotten list.

## Configuration

Use **Alfred Preferences → Workflows → JetBrains Projects (Rust) → Configure Workflow**
or `~/.config/alfred-jetbrains/config.toml`. A missing TOML file uses defaults.
`JB_CONFIG` selects another file; `jb config` prints effective settings.

| TOML key | Default / purpose |
| --- | --- |
| `show_stale` | `false`; include missing project directories |
| `hide_worktrees` | `true`; hide linked worktrees from normal search |
| `sort` | `"recent"`; also `oldest`, `name`, `name-desc`, `path` |
| `default_product` | `"idea"`; fallback IDE product ID or alias |
| `terminal` | `"Terminal"` in the CLI; the Alfred page defaults to `Ghostty` |
| `ignore` | `[]`; globs matched against project names and paths |
| `cache_seconds` | `0`; Alfred result cache lifetime in seconds, with `0` disabling caching |
| `settings_roots` | IDE settings search roots, listed below |
| `app_roots` | IDE application search roots, listed below |
| `pins_file` | `~/.config/alfred-jetbrains/pins.json` |
| `forgotten_file` | `~/.config/alfred-jetbrains/forgotten.json` |
| `workflow_name` | `"alfred-jetbrains-rust"`; installation directory name |

Default settings roots are `~/Library/Application Support/JetBrains`,
`~/Library/Application Support/Google`, and `~/Library/Preferences`.
Default app roots are `/Applications`, `~/Applications`, and
`~/Applications/JetBrains Toolbox`. Customize these for nonstandard installations.

Alfred passes `jb_terminal`, `jb_default_product`, `jb_show_stale`,
`jb_hide_worktrees`, `jb_sort`, `jb_settings_roots`, and `jb_app_roots` as environment
overrides. Root lists use colon separators. `jb_ignore` adds newline-separated
rules to the TOML list. Command options such as `search --sort name` can select a
sort order for one invocation.

Product overrides can change `preference_prefix`, `application_names`, `binaries`,
and `keywords`. For example:

```toml
[products.idea]
keywords = ["ij", "idea"]
```

Run `jb install` after changing keywords to regenerate the workflow. In this
example, `ij` becomes the independent Alfred keyword; both aliases still work
inside global queries. Pins and forgotten projects persist in their JSON files.

## Diagnostics and packaging

Terminal examples assume `jb` is on `PATH`; otherwise use its full path.
`jb search` emits Alfred JSON, while diagnostics go to stderr.

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

`doctor` reports discovered settings, IDE apps, project/worktree counts, hidden
project settings, and workflow installation status. Debug logging can also be
enabled with `JB_DEBUG=1` or Alfred's `alfred_debug` environment variable.

`install` installs or updates the workflow, including workflows imported from a
native package. `--workflow-name NAME` selects an explicit installation directory;
`--retain` leaves its existing binary in place. `export DIRECTORY` writes a portable
workflow directory without installing it. From the repository root,
`./tool/package_workflow.sh` packages the built binary as `.alfredworkflow`.
