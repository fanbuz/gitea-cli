# gitea-cli

[![Version](https://img.shields.io/badge/version-0.0.9-blue)](https://github.com/fanbuz/gitea-cli/releases/tag/v0.0.9)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](./LICENSE)

`gitea-cli` 是一个用 Rust 实现的本地命令行工具，用来把已经配置好的 Gitea MCP Server 暴露成可复用的 shell CLI。

它适合终端脚本、AI agent、自动化任务和日常排查场景：复用现有的 Codex MCP 配置，不额外维护一套 Gitea 登录流程，同时输出稳定 JSON，便于继续串接别的工具。

## Project Background

- 本项目是一个基于 Gitea 官方 MCP Server [`gitea/gitea-mcp`](https://gitea.com/gitea/gitea-mcp) 构建的 agent-friendly CLI 包装层。
- 本项目的 CLI 设计方式参考 OpenAI Codex 官方用例文档 [Create a CLI Codex can use](https://developers.openai.com/codex/use-cases/agent-friendly-clis)，重点放在 agent 友好的命令结构、稳定 JSON 输出、可脚本化和可验证性上。
- `gitea-cli` 自己不直接实现 Gitea API，也不重复造一套鉴权；它会读取 `~/.codex/config.toml` 中的 `mcp_servers.gitea.command`，把底层 MCP 能力整理成更适合人类和 agent 使用的命令面。
- 如果你使用的是官方 `gitea-mcp`，或本地已有 `gitea-mcp-server` 包装命令，只要该 MCP 配置可用，`gitea-cli` 就能直接复用。
- 后续规划会补一层 `gitea-cli` 自己的独立配置文件，让它在保留 Codex 兼容能力的同时，也能直接适配官方 brew 安装的 `gitea-mcp-server`。

## Features

- 复用 `~/.codex/config.toml` 里的 `gitea` MCP 配置
- 通过 stdio 与已配置的 Gitea MCP Server 通信，不重新发明鉴权
- 提供 `doctor` 健康检查，自动脱敏敏感参数
- 支持 `--fields` 对 JSON 输出做字段裁剪，降低 agent 调用时的 token 噪音
- 提供仓库、Issue、PR、Actions 等常见排查命令
- 提供 issue 创建、更新、评论、label 维护、milestone 管理与 time tracking 高层命令
- 提供 release 与 tag 的创建、删除高层命令，覆盖常见版本对象写操作
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

推荐通过 Homebrew 安装：

```bash
brew tap fanbuz/tap
brew install fanbuz/tap/gitea-cli
```

在已支持的平台上，Homebrew 会直接安装 GitHub Release 里的预编译二进制，不需要本地 Rust 编译环境。
当前预编译覆盖：`macOS arm64`、`macOS amd64`、`Linux x64`、`Windows x64`。
当目标平台暂时没有对应预编译包时，才会回退到源码构建。
当仓库配置了 `HOMEBREW_TAP_TOKEN` 后，每次推送新的 `v*` release tag 也会自动通知 `fanbuz/homebrew-tap` 更新 `gitea-cli` formula。

升级：

```bash
brew update
brew upgrade gitea-cli
```

如果你希望本地从源码安装：

```bash
git clone git@github.com:fanbuz/gitea-cli.git
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

如果你希望在 agent 或脚本场景里减少不必要的输出字段，也可以配合 `--fields`：

```bash
cargo run -- --json --fields kind,cli.version,issues doctor
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

只输出关心的关键字段：

```bash
gitea-cli --json --fields kind,result.parsed.id,result.parsed.title issues get --owner YOUR_ORG --repo YOUR_REPO --index 123
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

全局参数补充：

- `--json`
  输出 JSON，适合与 agent、脚本、`jq` 等组合使用。

- `--fields`
  仅在 JSON 输出中保留指定字段，支持逗号分隔和点号路径，例如 `kind,result.parsed.id,result.parsed.title`。

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

- `gitea-cli --json repos branch-create --owner YOUR_ORG --repo YOUR_REPO --branch feature/new-command --from main`
  基于指定已有分支创建新分支，适合在自动化脚本或 agent 协作流里快速补齐分支准备动作。

- `gitea-cli --json repos branch-delete --owner YOUR_ORG --repo YOUR_REPO --branch feature/new-command --yes`
  删除指定仓库分支，属于危险操作，必须显式传 `--yes`。

- `gitea-cli --json repos tree --owner YOUR_ORG --repo YOUR_REPO --ref main --recursive`
  读取某个仓库在指定 `ref` 下的文件树，可选择递归展开。

- `gitea-cli --json repos dir --owner YOUR_ORG --repo YOUR_REPO --ref main --path docs`
  读取某个仓库在指定 `ref` 下的目录内容；不传 `--path` 时默认读取仓库根目录。

- `gitea-cli --json repos file --owner YOUR_ORG --repo YOUR_REPO --path README.md --ref main --with-lines`
  读取某个仓库文件内容，可选 `--with-lines` 让底层返回逐行内容。

- `gitea-cli --json repos file-create --owner YOUR_ORG --repo YOUR_REPO --path docs/guide.md --branch main --message "create guide" --content "# guide"`
  在指定分支创建仓库文件，也可以改用 `--content-file ./guide.md` 从本地文件读取内容；可选 `--new-branch` 在新分支上提交。

- `gitea-cli --json repos file-update --owner YOUR_ORG --repo YOUR_REPO --path docs/guide.md --branch main --sha CURRENT_FILE_SHA --message "update guide" --content-file ./guide.md`
  更新已有仓库文件，必须显式传入当前文件 `--sha`，用于并发保护；内容同样支持 `--content` 与 `--content-file`。

- `gitea-cli --json repos file-delete --owner YOUR_ORG --repo YOUR_REPO --path docs/guide.md --branch main --sha CURRENT_FILE_SHA --message "delete guide" --yes`
  删除仓库文件，必须同时提供当前文件 `--sha` 和确认参数 `--yes`。

### Releases

- `gitea-cli --json releases list --owner YOUR_ORG --repo YOUR_REPO`
  列出某个仓库的 release 列表，适合做版本盘点和发版记录查询。

- `gitea-cli --json releases latest --owner YOUR_ORG --repo YOUR_REPO`
  读取某个仓库的最新 release，适合脚本里快速拿到最新发布版本。

- `gitea-cli --json releases get --owner YOUR_ORG --repo YOUR_REPO --id 12`
  按 release ID 读取单个 release 详情。

- `gitea-cli --json releases create --owner YOUR_ORG --repo YOUR_REPO --tag v0.0.7 --title "v0.0.7" --target main`
  创建一个 release，可附带 `--body`、`--draft` 和 `--pre-release`。

- `gitea-cli --json releases delete --owner YOUR_ORG --repo YOUR_REPO --id 12 --yes`
  删除一个 release，属于危险操作，必须显式传 `--yes`。

### Tags

- `gitea-cli --json tags list --owner YOUR_ORG --repo YOUR_REPO`
  列出某个仓库的 tag，适合排查版本标记和发布点。

- `gitea-cli --json tags get --owner YOUR_ORG --repo YOUR_REPO --tag YOUR_TAG`
  按 tag 名读取单个 tag 详情。

- `gitea-cli --json tags create --owner YOUR_ORG --repo YOUR_REPO --tag v0.0.7 --target main`
  创建一个 tag，可选附带 `--message` 用于 annotated tag。

- `gitea-cli --json tags delete --owner YOUR_ORG --repo YOUR_REPO --tag v0.0.7 --yes`
  删除一个 tag，属于危险操作，必须显式传 `--yes`。

### Commits

- `gitea-cli --json commits list --owner YOUR_ORG --repo YOUR_REPO`
  列出某个仓库的提交历史，支持 `--sha`、`--path` 和分页参数做过滤。

- `gitea-cli --json commits get --owner YOUR_ORG --repo YOUR_REPO --sha COMMIT_SHA`
  按 commit SHA 读取单个提交详情。

### Issues

- `gitea-cli --json issues list --owner YOUR_ORG --repo YOUR_REPO --state open`
  列出某个仓库的 Issue，支持状态、标签、时间范围和分页参数。

- `gitea-cli --json issues get --owner YOUR_ORG --repo YOUR_REPO --index 123`
  读取单个 Issue 的详情，适合做定点排查或脚本抓取。

- `gitea-cli --json issues comments --owner YOUR_ORG --repo YOUR_REPO --index 123`
  读取某个 Issue 的评论列表，方便查看讨论上下文。

- `gitea-cli --json issues comments --owner YOUR_ORG --repo YOUR_REPO --index 123 --comment-id 88 --comment-id 99`
  读取某个 Issue 的评论列表，并在本地仅保留指定评论，适合 agent 或脚本做定点提取。

- `gitea-cli --json issues search --query "exact phrase" --owner YOUR_ORG`
  按关键字跨仓库搜索 Issue，可结合 `--owner`、`--state` 等参数缩小范围。

- `gitea-cli --json issues create --owner YOUR_ORG --repo YOUR_REPO --title "Need fix"`
  创建一个 issue，可附带正文、assignee、label IDs、milestone、关联分支和截止时间。

- `gitea-cli --json issues update --owner YOUR_ORG --repo YOUR_REPO --index 123 --state closed`
  更新一个已有 issue，可修改标题、正文、状态、labels、milestone、关联分支和截止时间。

- `gitea-cli --json issues comment-add --owner YOUR_ORG --repo YOUR_REPO --index 123 --body "follow up"`
  为 issue 新增一条评论。

- `gitea-cli --json issues comment-edit --owner YOUR_ORG --repo YOUR_REPO --index 123 --comment-id 88 --body "edited"`
  编辑 issue 上已有的一条评论。

- `gitea-cli --json issues labels --owner YOUR_ORG --repo YOUR_REPO --index 123`
  读取 issue 当前绑定的 labels。

- `gitea-cli --json issues labels-add --owner YOUR_ORG --repo YOUR_REPO --index 123 --label-id 1 --label-id 2`
  为 issue 追加一组 labels。

- `gitea-cli --json issues label-remove --owner YOUR_ORG --repo YOUR_REPO --index 123 --label-id 2 --yes`
  从 issue 上删除单个 label，属于危险操作，必须显式传 `--yes`。

- `gitea-cli --json issues labels-replace --owner YOUR_ORG --repo YOUR_REPO --index 123 --label-id 4 --label-id 5`
  用一组 labels 替换 issue 当前的 labels 集合。

- `gitea-cli --json issues labels-clear --owner YOUR_ORG --repo YOUR_REPO --index 123 --yes`
  清空 issue 当前全部 labels，属于危险操作，必须显式传 `--yes`。

### Issue Time

- `gitea-cli --json issues time list --owner YOUR_ORG --repo YOUR_REPO --index 123`
  读取 issue 当前的 time tracking 记录。

- `gitea-cli --json issues time start --owner YOUR_ORG --repo YOUR_REPO --index 123`
  为 issue 启动 stopwatch。

- `gitea-cli --json issues time stop --owner YOUR_ORG --repo YOUR_REPO --index 123`
  停止 issue 当前 stopwatch。

- `gitea-cli --json issues time reset-stopwatch --owner YOUR_ORG --repo YOUR_REPO --index 123 --yes`
  重置 issue 当前 stopwatch，属于危险操作，必须显式传 `--yes`。

- `gitea-cli --json issues time add --owner YOUR_ORG --repo YOUR_REPO --index 123 --seconds 120`
  为 issue 增加一条指定秒数的 tracked time 记录。

- `gitea-cli --json issues time delete --owner YOUR_ORG --repo YOUR_REPO --index 123 --id 77 --yes`
  删除一条 issue time 记录，属于危险操作，必须显式传 `--yes`。

### Labels

- `gitea-cli --json labels repo-list --owner YOUR_ORG --repo YOUR_REPO`
  列出某个仓库下的全部 labels，适合在 issue 编排前先确认可用 label 集合。

- `gitea-cli --json labels repo-get --owner YOUR_ORG --repo YOUR_REPO --id 9`
  按 ID 读取单个仓库 label 的详情。

- `gitea-cli --json labels repo-create --owner YOUR_ORG --repo YOUR_REPO --name bug --color '#ff0000'`
  在仓库下创建一个 label，可附带描述和 archived 标记。

- `gitea-cli --json labels repo-edit --owner YOUR_ORG --repo YOUR_REPO --id 9 --name urgent`
  更新仓库 label，可按需修改名称、颜色、描述和 archived 状态。

- `gitea-cli --json labels repo-delete --owner YOUR_ORG --repo YOUR_REPO --id 9 --yes`
  删除一个仓库 label，属于危险操作，必须显式传 `--yes`。

- `gitea-cli --json labels org-list --org YOUR_ORG`
  列出组织级 labels，适合查看组织默认标签池。

- `gitea-cli --json labels org-create --org YOUR_ORG --name backend --color '#0055cc'`
  在组织下创建一个 label，可附带描述和 exclusive 标记。

- `gitea-cli --json labels org-edit --org YOUR_ORG --id 7 --name frontend`
  更新组织 label，可按需修改名称、颜色、描述和 exclusive 状态。

- `gitea-cli --json labels org-delete --org YOUR_ORG --id 7 --yes`
  删除一个组织 label，属于危险操作，必须显式传 `--yes`。

### Milestones

- `gitea-cli --json milestones list --owner YOUR_ORG --repo YOUR_REPO --state open`
  列出仓库 milestones，可结合状态、名称和分页参数筛选。

- `gitea-cli --json milestones get --owner YOUR_ORG --repo YOUR_REPO --id 3`
  按 ID 读取单个 milestone 的详情。

- `gitea-cli --json milestones create --owner YOUR_ORG --repo YOUR_REPO --title v0.0.6`
  创建一个 milestone，可附带描述和截止时间。

- `gitea-cli --json milestones edit --owner YOUR_ORG --repo YOUR_REPO --id 3 --title v0.0.6`
  更新已有 milestone，可修改标题、描述、状态和截止时间。

- `gitea-cli --json milestones delete --owner YOUR_ORG --repo YOUR_REPO --id 3 --yes`
  删除一个 milestone，属于危险操作，必须显式传 `--yes`。

### Pull Requests

- `gitea-cli --json pulls list --owner YOUR_ORG --repo YOUR_REPO --state open`
  列出某个仓库的 Pull Request，适合做待合并队列和状态排查。

- `gitea-cli --json pulls create --owner YOUR_ORG --repo YOUR_REPO --head feature-branch --base main --title "Add feature"`
  创建一个 Pull Request，可附带正文、label IDs、draft 标记和截止时间。

- `gitea-cli --json pulls update --owner YOUR_ORG --repo YOUR_REPO --index 12 --title "New title"`
  更新已有 Pull Request 的主体信息，可修改标题、正文、状态、目标分支、assignee、labels、milestone、截止时间、draft 状态和 maintainer 编辑权限。

- `gitea-cli --json pulls merge --owner YOUR_ORG --repo YOUR_REPO --index 12 --merge-style squash`
  合并一个 Pull Request，可控制 merge style、删除分支、强制合并、检查通过后自动合并和预期 head commit。

- `gitea-cli --json pulls get --owner YOUR_ORG --repo YOUR_REPO --index 12`
  读取单个 Pull Request 的详情，包括标题、状态、分支信息等。

- `gitea-cli --json pulls diff --owner YOUR_ORG --repo YOUR_REPO --index 12`
  读取单个 Pull Request 的 diff 内容，用于脚本化审查或上下文提取。

- `gitea-cli --json pulls review-comments --owner YOUR_ORG --repo YOUR_REPO --index 12 --review-id 7`
  读取某个 Pull Request review 下的评论列表，方便补齐评审上下文。

- `gitea-cli --json pulls review-comments --owner YOUR_ORG --repo YOUR_REPO --index 12 --review-id 7 --comment-id 101`
  读取某个 Pull Request review 下的评论列表，并在本地仅保留指定评论。

- `gitea-cli --json pulls reviews --owner YOUR_ORG --repo YOUR_REPO --index 12`
  读取某个 Pull Request 的 review 列表，适合查看当前评审流转情况。

- `gitea-cli --json pulls review-get --owner YOUR_ORG --repo YOUR_REPO --index 12 --review-id 7`
  读取单个 Pull Request review 的详情。

- `gitea-cli --json pulls reviewers-add --owner YOUR_ORG --repo YOUR_REPO --index 12 --reviewer alice --reviewer bob`
  为指定 Pull Request 添加 reviewer。

- `gitea-cli --json pulls reviewers-remove --owner YOUR_ORG --repo YOUR_REPO --index 12 --reviewer alice`
  为指定 Pull Request 移除 reviewer。

- `gitea-cli --json pulls review-create --owner YOUR_ORG --repo YOUR_REPO --index 12 --body "needs review"`
  创建一个 review 壳子，可选附带 `--commit-id`。

- `gitea-cli --json pulls review-submit --owner YOUR_ORG --repo YOUR_REPO --index 12 --review-id 7 --state APPROVED --body "looks good"`
  提交一个已有 review，状态严格限制为 `APPROVED`、`REQUEST_CHANGES` 或 `COMMENT`。

- `gitea-cli --json pulls review-delete --owner YOUR_ORG --repo YOUR_REPO --index 12 --review-id 7 --yes`
  删除一个 review，属于危险操作，必须显式传 `--yes`。

- `gitea-cli --json pulls review-dismiss --owner YOUR_ORG --repo YOUR_REPO --index 12 --review-id 7 --message "stale review"`
  撤销一个 review，并要求显式说明原因。

### Actions

- `gitea-cli --json actions workflows --owner YOUR_ORG --repo YOUR_REPO`
  列出某个仓库的 Actions workflows。

- `gitea-cli --json actions runs --owner YOUR_ORG --repo YOUR_REPO`
  列出某个仓库的 Actions runs，可结合状态和分页参数筛选。

- `gitea-cli --json actions jobs --owner YOUR_ORG --repo YOUR_REPO --run-id 456`
  读取某次 run 下的 jobs；如果不传 `--run-id`，则读取仓库级 job 列表。

- `gitea-cli --json actions log-preview --owner YOUR_ORG --repo YOUR_REPO --job-id 789`
  读取某个 job 的日志预览，适合先快速看失败摘要，不必先下载完整日志。

- `gitea-cli --json actions dispatch --owner YOUR_ORG --repo YOUR_REPO --workflow-id release.yml --ref main --inputs '{"env":"prod"}'`
  触发指定 workflow 运行，支持以内联 JSON 或 `@file` 方式传入 inputs。

- `gitea-cli --json actions cancel --owner YOUR_ORG --repo YOUR_REPO --run-id 456`
  取消指定 workflow run。

- `gitea-cli --json actions rerun --owner YOUR_ORG --repo YOUR_REPO --run-id 456`
  重跑指定 workflow run。

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

- [x] 分支写操作
  已提供 `repos branch-create`、`repos branch-delete`。

- [x] Release / Tag / Commit
  已提供 `releases list/latest/get/create/delete`、`tags list/get/create/delete`、`commits list/get`。

- [x] 文件与目录内容管理
  已提供 `repos dir`、`repos file`、`repos file-create`、`repos file-update`、`repos file-delete`。

- [x] Issue 读取与写操作
  已提供 `issues list/get/comments/search/create/update/comment-add/comment-edit/labels/labels-add/label-remove/labels-replace/labels-clear`，其中 `issues comments` 支持通过可重复 `--comment-id` 过滤评论子集。

- [x] Pull Request 主体读写操作
  已提供 `pulls list/create/update/merge/get/diff/review-comments/reviews/review-get`。

- [x] Pull Request 评审与 reviewer 管理
  已提供 `pulls reviewers-add/reviewers-remove/review-create/review-submit/review-delete/review-dismiss`。

- [x] Actions 只读排查路径
  已提供 `actions workflows`、`actions runs`、`actions jobs`、`actions log-preview`。

- [x] Actions 执行控制
  已提供 `actions dispatch`、`actions cancel`、`actions rerun`。

- [ ] Actions 配置管理
  暂未单独封装 secrets、variables 等高层命令。

- [x] Labels
  已提供 `labels repo-list/repo-get/repo-create/repo-edit/repo-delete/org-list/org-create/org-edit/org-delete`。

- [x] Milestones
  已提供 `milestones list/get/create/edit/delete`。

- [ ] Wiki
  暂未单独封装 wiki page 与 revision 相关高层命令。

- [x] Time Tracking
  已提供 `issues time list/start/stop/reset-stopwatch/add/delete`，覆盖 issue 级 time tracking 常用工作流。

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

- 高层命令以排查与受控写操作为主
- `mcp call` 是原始逃生口，理论上可以调用更多写操作工具
- `issues label-remove`、`issues labels-clear`、`issues time reset-stopwatch`、`issues time delete`、`labels repo-delete`、`labels org-delete`、`milestones delete`、`releases delete`、`tags delete` 这类危险操作必须显式传 `--yes`
- 其余写命令保持可脚本化，不额外弹确认

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
  在 `Linux x64`、`macOS arm64`、`macOS amd64`、`Windows x64` 上编译 `release` 二进制，并把压缩包上传为 workflow artifacts。

- 推送 `v*` 标签时，会执行 `.github/workflows/release.yml`
  在 `Linux x64`、`macOS arm64`、`macOS amd64`、`Windows x64` 上重新构建二进制，并自动创建或更新对应的 GitHub Release，上传可下载的压缩包。

- 如果仓库配置了 `HOMEBREW_TAP_TOKEN`
  release workflow 会在 GitHub Release 成功后，通过 `repository_dispatch` 通知 `fanbuz/homebrew-tap` 拉取最新 release 元数据并更新 `gitea-cli` formula。

- 如果某个历史标签早于 workflow 提交，比如已经存在的 `v0.0.1`
  可以在 GitHub Actions 页面手动触发 `Release Binaries`，并把 `tag` 输入设为对应版本号，补发 release 产物。

- 当前产物格式：
  Linux / macOS 为 `tar.gz`
  Windows 为 `zip`

## Roadmap

- 补充 wiki、PR 写操作、Actions 写操作等高层命令
- 增加更完整的分页和过滤支持
- 在未来引入写命令前补充更清晰的安全护栏
- 增加 `gitea-cli` 独立配置层
  计划支持 `~/.config/gitea-cli/config.toml` 一类的本地配置文件，并采用 `gitea-cli 自身配置 > ~/.codex/config.toml > 默认 gitea-mcp-server` 的回退顺序。
  目标是让 `gitea-cli` 既能复用 Codex MCP 配置，也能在官方 `gitea-mcp-server` brew 安装场景下独立工作。

## Contributing

欢迎提交 Issue 和 Pull Request。开始之前请先阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。

## Security

安全问题请参考 [SECURITY.md](SECURITY.md)。

## License

本项目使用 [MIT License](LICENSE)。
