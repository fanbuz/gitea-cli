# gitea-cli

[![Version](https://img.shields.io/badge/version-0.0.2-blue)](https://github.com/Mashull/gitea-cli/releases/tag/v0.0.2)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](./LICENSE)

`gitea-cli` 是一个用 Rust 实现的本地命令行工具，用来把已经配置好的 Gitea MCP Server 暴露成可复用的 shell CLI。

它适合终端脚本、AI agent、自动化任务和日常排查场景：复用现有的 Codex MCP 配置，不额外维护一套 Gitea 登录流程，同时输出稳定 JSON，便于继续串接别的工具。

## Project Background

- 本项目是一个基于 Gitea 官方 MCP Server [`gitea/gitea-mcp`](https://gitea.com/gitea/gitea-mcp) 构建的 agent-friendly CLI 包装层。
- 本项目的 CLI 设计方式参考 OpenAI Codex 官方用例文档 [Create a CLI Codex can use](https://developers.openai.com/codex/use-cases/agent-friendly-clis)，重点放在 agent 友好的命令结构、稳定 JSON 输出、可脚本化和可验证性上。
- `gitea-cli` 自己不直接实现 Gitea API，也不重复造一套鉴权；它会读取 `~/.codex/config.toml` 中的 `mcp_servers.gitea.command`，把底层 MCP 能力整理成更适合人类和 agent 使用的命令面。
- 如果你使用的是官方 `gitea-mcp`，或本地已有 `gitea-mcp-server` 包装命令，只要该 MCP 配置可用，`gitea-cli` 就能直接复用。

## Features

- 复用 `~/.codex/config.toml` 里的 `gitea` MCP 配置
- 通过 stdio 与已配置的 Gitea MCP Server 通信，不重新发明鉴权
- 提供 `doctor` 健康检查，自动脱敏敏感参数
- 提供仓库、Issue、PR、Actions 等常见只读命令
- 当 MCP 返回单条 JSON 文本内容时，自动补充 `result.parsed`
- 保留 `mcp call` 原始出口，方便覆盖未封装的工具

## Requirements

- Rust toolchain
- 可执行的 Gitea MCP Server，例如官方项目 [`gitea/gitea-mcp`](https://gitea.com/gitea/gitea-mcp)
- 已配置好的 `~/.codex/config.toml`

示例配置：

```toml
[mcp_servers.gitea]
type = "stdio"
command = "gitea-mcp"
args = ["-t", "stdio", "--host", "https://your-gitea.example.com"]

[mcp_servers.gitea.env]
GITEA_ACCESS_TOKEN = "YOUR_TOKEN"
```

如果你的环境里使用的是 `gitea-mcp-server` 或其他包装命令，也可以直接填在 `command` 字段中。

## Installation

本地安装：

```bash
git clone git@github.com:Mashull/gitea-cli.git
cd gitea-cli
make install-local
```

安装完成后可执行文件位于：

```bash
~/.local/bin/gitea-cli
```

如果只想临时运行，也可以直接：

```bash
cargo run -- --json doctor
```

## Quick Start

检查配置和 MCP 连通性：

```bash
gitea-cli --json doctor
```

列出 MCP 工具：

```bash
gitea-cli --json tools list
```

读取当前登录用户：

```bash
gitea-cli --json me
```

读取一个 Issue：

```bash
gitea-cli --json issues get --owner YOUR_ORG --repo YOUR_REPO --index 123
```

解析 Gitea URL：

```bash
gitea-cli --json resolve issue https://your-gitea.example.com/YOUR_ORG/YOUR_REPO/issues/123
```

直接调用底层 MCP 工具：

```bash
gitea-cli --json mcp call issue_read --params '{"owner":"YOUR_ORG","repo":"YOUR_REPO","index":123,"method":"get"}'
```

## Command Surface

### Health

- `gitea-cli --json doctor`
  检查 `~/.codex/config.toml`、MCP 命令是否在 `PATH` 中、MCP 能否启动，以及当前可用工具数量。

- `gitea-cli --json tools list`
  列出当前 Gitea MCP Server 暴露出来的全部工具，便于确认高层命令之外还能直接调用什么。

### Identity

- `gitea-cli --json me`
  读取当前鉴权用户信息，适合快速确认 CLI 复用了哪个 Gitea 身份。

- `gitea-cli --json orgs list`
  列出当前用户可访问的组织，适合补全后续 `--owner` 参数。

### Repositories

- `gitea-cli --json repos list --owner YOUR_ORG`
  列出指定组织下的仓库；如果不传 `--owner`，则读取当前用户自己的仓库列表。

- `gitea-cli --json repos branches --owner YOUR_ORG --repo YOUR_REPO`
  列出某个仓库的分支，用于排查默认分支、发布分支或临时分支状态。

- `gitea-cli --json repos tree --owner YOUR_ORG --repo YOUR_REPO --ref main --recursive`
  读取某个仓库在指定 `ref` 下的文件树，可选择递归展开。

### Issues

- `gitea-cli --json issues list --owner YOUR_ORG --repo YOUR_REPO --state open`
  列出某个仓库的 Issue，支持状态、标签、时间范围和分页参数。

- `gitea-cli --json issues get --owner YOUR_ORG --repo YOUR_REPO --index 123`
  读取单个 Issue 的详情，适合做定点排查或脚本抓取。

- `gitea-cli --json issues comments --owner YOUR_ORG --repo YOUR_REPO --index 123`
  读取某个 Issue 的评论列表，方便查看讨论上下文。

- `gitea-cli --json issues search --query "exact phrase" --owner YOUR_ORG`
  按关键字跨仓库搜索 Issue，可结合 `--owner`、`--state` 等参数缩小范围。

### Pull Requests

- `gitea-cli --json pulls list --owner YOUR_ORG --repo YOUR_REPO --state open`
  列出某个仓库的 Pull Request，适合做待合并队列和状态排查。

- `gitea-cli --json pulls get --owner YOUR_ORG --repo YOUR_REPO --index 12`
  读取单个 Pull Request 的详情，包括标题、状态、分支信息等。

- `gitea-cli --json pulls diff --owner YOUR_ORG --repo YOUR_REPO --index 12`
  读取单个 Pull Request 的 diff 内容，用于脚本化审查或上下文提取。

### Actions

- `gitea-cli --json actions workflows --owner YOUR_ORG --repo YOUR_REPO`
  列出某个仓库的 Actions workflows。

- `gitea-cli --json actions runs --owner YOUR_ORG --repo YOUR_REPO`
  列出某个仓库的 Actions runs，可结合状态和分页参数筛选。

- `gitea-cli --json actions jobs --owner YOUR_ORG --repo YOUR_REPO --run-id 456`
  读取某次 run 下的 jobs；如果不传 `--run-id`，则读取仓库级 job 列表。

- `gitea-cli --json actions log-preview --owner YOUR_ORG --repo YOUR_REPO --job-id 789`
  读取某个 job 的日志预览，适合先快速看失败摘要，不必先下载完整日志。

### URL Resolve

- `gitea-cli --json resolve repo <repo-url>`
  把仓库 URL 解析成稳定的 `owner/repo` 坐标。

- `gitea-cli --json resolve issue <issue-url>`
  把 Issue URL 解析成 `owner/repo/index`，便于继续拼装 API 或 CLI 调用。

- `gitea-cli --json resolve pull <pull-url>`
  把 Pull Request URL 解析成 `owner/repo/index`。

### Raw MCP Escape Hatch

- `gitea-cli --json mcp call <tool-name> --params '{"key":"value"}'`
  直接调用底层 MCP 工具，并以内联 JSON 方式传参。

- `gitea-cli --json mcp call <tool-name> --params @params.json`
  直接调用底层 MCP 工具，并从 JSON 文件读取参数，适合复杂请求体或脚本场景。

## Official MCP Coverage Checklist

下面的清单是基于 Gitea 官方 MCP Server [`gitea/gitea-mcp`](https://gitea.com/gitea/gitea-mcp) 当前公开工具列表整理的。这里的“已实现”特指已经有独立的高层 CLI 子命令；“未单独封装”表示底层官方 MCP 通常已有能力，但当前仍建议通过 `gitea-cli --json mcp call ...` 访问。

- [x] 用户信息
  已提供 `me`、`orgs list`。

- [x] 仓库基础读取
  已提供 `repos list`、`repos branches`、`repos tree`。

- [ ] 仓库写操作与高级仓库管理
  暂未单独封装 `create repo`、`fork repo`、`search repos` 等高层命令。

- [ ] 分支写操作
  暂未单独封装 `create branch`、`delete branch`。

- [ ] Release / Tag / Commit
  暂未单独封装 release、tag、commit 相关高层命令。

- [ ] 文件与目录内容管理
  暂未单独封装文件读取、目录读取、创建文件、更新文件、删除文件等高层命令。

- [x] Issue 只读操作
  已提供 `issues list`、`issues get`、`issues comments`、`issues search`。

- [ ] Issue 写操作
  暂未单独封装创建 Issue、编辑 Issue、创建评论、编辑评论等高层命令。

- [x] Pull Request 只读操作
  已提供 `pulls list`、`pulls get`、`pulls diff`。

- [ ] Pull Request 审查与写操作
  暂未单独封装创建 PR、增删 reviewer、review、dismiss、merge 等高层命令。

- [x] Actions 只读排查路径
  已提供 `actions workflows`、`actions runs`、`actions jobs`、`actions log-preview`。

- [ ] Actions 写操作与配置管理
  暂未单独封装 workflow dispatch、cancel/rerun、secrets、variables 等高层命令。

- [ ] Labels
  暂未单独封装 repo/org labels 相关高层命令。

- [ ] Milestones
  暂未单独封装 milestone 相关高层命令。

- [ ] Wiki
  暂未单独封装 wiki page 与 revision 相关高层命令。

- [ ] Time Tracking
  暂未单独封装 time tracking 相关高层命令。

- [x] 官方 MCP 原始能力透传
  已提供 `mcp call`，可直接调用底层官方 MCP 工具，作为高层命令尚未覆盖时的兜底出口。

## Output Contract

成功时统一返回 JSON，对外字段尽量稳定：

- `ok`: 是否成功
- `kind`: 响应类别，例如 `doctor`、`tools_list`、`tool_call`、`resolve`
- `result`: 原始 MCP 返回结果
- `result.parsed`: 当结果是单条 JSON 文本内容时自动补充的解析结果

失败时返回：

- `ok: false`
- `error`: 可读的错误信息

## Safety

- 高层命令默认偏只读
- `mcp call` 是原始逃生口，理论上可以调用写操作工具
- 涉及写操作时，建议显式确认后再执行

## Development

运行测试：

```bash
cargo test
```

格式化代码：

```bash
cargo fmt
```

安装本地命令：

```bash
make install-local
```

## GitHub Actions Build And Release

- 推送到 `main`、提交 Pull Request，或手动触发 workflow 时，会执行 `.github/workflows/build.yml`
  在 Linux、macOS、Windows 上编译 `release` 二进制，并把压缩包上传为 workflow artifacts。

- 推送 `v*` 标签时，会执行 `.github/workflows/release.yml`
  在三种操作系统上重新构建二进制，并自动创建或更新对应的 GitHub Release，上传可下载的压缩包。

- 如果某个历史标签早于 workflow 提交，比如已经存在的 `v0.0.1`
  可以在 GitHub Actions 页面手动触发 `Release Binaries`，并把 `tag` 输入设为对应版本号，补发 release 产物。

- 当前产物格式：
  Linux / macOS 为 `tar.gz`
  Windows 为 `zip`

## Roadmap

- 补充 labels、milestones、releases、wiki 等高层命令
- 增加更完整的分页和过滤支持
- 在未来引入写命令前补充更清晰的安全护栏

## Contributing

欢迎提交 Issue 和 Pull Request。开始之前请先阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。

## Security

安全问题请参考 [SECURITY.md](SECURITY.md)。

## License

本项目使用 [MIT License](LICENSE)。
