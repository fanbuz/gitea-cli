# gitea-cli

[![Version](https://img.shields.io/badge/version-1.10.0-blue)](https://github.com/fanbuz/gitea-cli/releases/tag/v1.10.0)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](./LICENSE)

`gitea-cli` 是一个用 Rust 实现的本地命令行工具，用来把 Gitea MCP Server 暴露成可复用的 shell CLI。

它适合终端脚本、AI agent、自动化任务和日常排查场景：通过自己的配置文件管理 Gitea 实例和认证来源，同时输出稳定 JSON，便于继续串接别的工具。

## Project Background

- 本项目是一个基于 Gitea 官方 MCP Server [`gitea/gitea-mcp`](https://gitea.com/gitea/gitea-mcp) 构建的 agent-friendly CLI 包装层。
- 项目重点放在 agent 友好的命令结构、稳定 JSON 输出、可脚本化和可验证性上。
- `gitea-cli` 自己不直接实现 Gitea API；它读取 `~/.config/gitea-cli/config.toml`，解析 profile、host、认证来源和 MCP server 启动参数，再通过 stdio 调用底层 MCP 能力。
- 如果你使用的是官方 `gitea-mcp`，或本地已有 `gitea-mcp-server` 包装命令，只要该命令可执行，`gitea-cli` 就能通过配置文件启动它。

## Features

- 通过 `~/.config/gitea-cli/config.toml` 管理 Gitea 实例、profile 与认证来源
- 提供 `config path/init/show/validate` 配置管理命令
- 通过 stdio 与已配置的 Gitea MCP Server 通信，不重新发明鉴权
- 提供 `doctor` 健康检查，自动脱敏敏感参数
- 支持 `--fields` 对 JSON 输出做字段裁剪，降低 agent 调用时的 token 噪音
- 提供仓库、Issue、PR、Actions 等常见排查命令
- 提供 issue 创建、更新、评论、label 维护、milestone 管理与 time tracking 高层命令
- 提供 release 与 tag 的创建、删除高层命令，覆盖常见版本对象写操作
- 当 MCP 返回单条 JSON 文本内容时，自动补充 `result.parsed`
- 保留 `mcp call` 原始出口，方便覆盖未封装的工具

## Gitea MCP 1.10.0 Compatibility

`gitea-cli 1.10.0` 对齐 [Gitea MCP Server 1.10.0](https://gitea.com/gitea/gitea-mcp/releases/tag/v1.10.0)，在原有能力基础上新增：

- `commits compare`：比较两个分支、tag 或 SHA，返回 head 相对 base 新增的提交及涉及的文件。
- `commits get --include-diff`：读取单个提交时附带 unified diff；默认保持原有输出。
- `commits status` / `status-create`：读取汇总状态及各 context 的最新结果，或创建提交状态。
- `pulls commits` / `by-branches` / `by-commit`：读取 PR 提交列表，或按分支对、提交定位 PR。
- `pulls reviewer-candidates` / `issues assignee-candidates`：读取仓库可审查、可指派的用户，无需 issue 或 PR 编号。
- `pulls create --assignee/--milestone/--reviewer/--team-reviewer`：在创建时传递人员、团队与里程碑。
- `actions rerun-job` / `rerun-failed-jobs` / `approve` / `workflow-enable` / `workflow-disable`：单 job 重跑、失败 jobs 重跑、run 审批及 workflow 启停。**`approve` 要求目标 Gitea 1.28+**。
- `issues comment-edit` 改用 `comment_id`；MCP 1.10.0 仍接受旧的 `commentID`。
- `labels repo-create --exclusive` / `repo-edit --exclusive true|false`：创建或修改仓库互斥标签。

```bash
gitea-cli commits compare --owner OWNER --repo REPO --base main --head feature/docs
gitea-cli commits get --owner OWNER --repo REPO --sha COMMIT_SHA --include-diff
gitea-cli commits status --owner OWNER --repo REPO --ref main
gitea-cli commits status-create --owner OWNER --repo REPO --sha COMMIT_SHA --state success --context ci/build --description "Build passed" --target-url https://ci.example.com/build/123
gitea-cli pulls commits --owner OWNER --repo REPO --index 12 --page 1 --page-size 30
gitea-cli pulls by-branches --owner OWNER --repo REPO --base main --head feature/docs
gitea-cli pulls by-commit --owner OWNER --repo REPO --sha COMMIT_SHA
gitea-cli pulls reviewer-candidates --owner OWNER --repo REPO
gitea-cli issues assignee-candidates --owner OWNER --repo REPO
gitea-cli pulls create --owner OWNER --repo REPO --head feature/docs --base main --title "Update docs" --assignee alice --milestone 2 --reviewer bob --team-reviewer maintainers
gitea-cli actions rerun-job --owner OWNER --repo REPO --run-id 123 --job-id 456
gitea-cli actions rerun-failed-jobs --owner OWNER --repo REPO --run-id 123
gitea-cli actions approve --owner OWNER --repo REPO --run-id 123
gitea-cli actions workflow-enable --owner OWNER --repo REPO --workflow-id build.yml
gitea-cli actions workflow-disable --owner OWNER --repo REPO --workflow-id build.yml
gitea-cli labels repo-create --owner OWNER --repo REPO --name priority/high --color '#ff0000' --exclusive
gitea-cli labels repo-edit --owner OWNER --repo REPO --id 6 --exclusive false
```

提交状态支持 `pending`、`success`、`error`、`failure`、`warning`；读取状态的 `--ref` 接受 SHA、分支或 tag。`rerun-job` 必须同时提供 `--run-id` 和 `--job-id`；workflow 启停使用 ID 或文件名，无需 run ID。PR 创建的人员与团队参数可重复传入，未指定的可选字段不发送。`by-commit` 依赖目标 Gitea 已记录的提交与 PR 关联；未记录关联的提交会返回不存在。`by-branches` 直接返回服务端选出的 PR；同一分支对重复使用时，可能返回已关闭的历史 PR。

MCP 1.10.0 修复了 PR 创建时忽略 assignees、milestone、reviewers，以及仓库标签创建/编辑忽略 `exclusive` 的问题；CLI 为仓库标签补充了对应参数。服务端还将日志统一输出到 stderr，修复 `-d` 调试日志干扰 stdio 消息的问题。仓库列表返回精简对象，CLI 透明保留服务端结果；依赖完整字段的脚本应核对实际输出。上述新增命令及 `comment_id` 参数需要 MCP 1.10.0；目标 Gitea 版本或权限不足时，CLI 会返回服务端错误。

## Gitea MCP 1.9.0 Compatibility

`gitea-cli 1.9.0` 对齐 [Gitea MCP Server 1.9.0](https://gitea.com/gitea/gitea-mcp/releases/tag/v1.9.0)，同时覆盖 1.8.x 的新增工具能力：

- `orgs create`：创建组织，支持完整名称、描述、网站与可见性。
- `projects`：读取与管理用户、组织及仓库项目、列和项目 issue；**目标 Gitea 实例必须为 v28+**。
- `repos file --start-line/--end-line`：读取指定行区间；`file-update --edits`：精确文本替换。
- `repos files-write`：单次提交多个文件，支持创建、更新、编辑、重命名和删除；包含删除时必须传 `--yes`。
- `issues comment-get/blocked-by/blocking/dependency-add/dependency-remove`：单条评论读取与 issue 依赖。
- `pulls comments/status`：普通讨论评论及 CI 状态，可通过 `--wait-seconds 0..45` 等待。
- `actions runs --branch/--head-sha`：按分支与完整提交 SHA 筛选。
- 配置可透传 mTLS、自定义 CA、额外 HTTP 请求头和 Unix socket 参数。OAuth 授权与 HTTP healthcheck 属于服务端 HTTP 模式，CLI 仍使用 stdio。

### 1.9.0 命令示例

```bash
gitea-cli orgs create --username my-team --full-name "My Team" --visibility private
gitea-cli repos file --owner OWNER --repo REPO --path README.md --start-line 10 --end-line 30 --with-lines
gitea-cli repos file-update --owner OWNER --repo REPO --path README.md --branch main --sha CURRENT_SHA --message "Update title" --edits '[{"old_string":"Old title","new_string":"New title"}]'
gitea-cli repos files-write --owner OWNER --repo REPO --branch main --new-branch feature/docs --message "Update docs" --files @files.json --yes
gitea-cli issues comment-get --owner OWNER --repo REPO --comment-id 123
gitea-cli issues blocked-by --owner OWNER --repo REPO --index 12
gitea-cli issues blocking --owner OWNER --repo REPO --index 12
gitea-cli issues dependency-add --owner OWNER --repo REPO --index 12 --dependency-type blocked_by --related-issue-number 8
gitea-cli issues dependency-remove --owner OWNER --repo REPO --index 12 --dependency-type blocked_by --related-issue-number 8 --yes
gitea-cli pulls comments --owner OWNER --repo REPO --index 12
gitea-cli pulls status --owner OWNER --repo REPO --index 12 --wait-seconds 30
gitea-cli actions runs --owner OWNER --repo REPO --branch main --head-sha FULL_COMMIT_SHA
```

`--edits` 与 `--files` 接受 JSON 或 `@本地文件`。内容是原始文本，不是 base64；`content` 与 `edits` 互斥。`files.json` 示例：

```json
[
  {"path":"docs/new.md","content":"# New document\n"},
  {"path":"README.md","edits":[{"old_string":"Old title","new_string":"New title","replace_all":false}]},
  {"path":"docs/renamed.md","from_path":"docs/old.md","sha":"SOURCE_SHA"},
  {"path":"docs/obsolete.md","delete":true,"sha":"CURRENT_SHA"}
]
```

已有文件完整替换要提供 `sha`，否则表示创建；重命名使用源文件 SHA，删除使用当前文件 SHA。批量 `edits` 由服务端读取当前内容和 SHA，可另传 `sha` 做预期版本校验；原有 `file-update` 始终要求显式 `--sha`。替换文本默认必须唯一匹配，重复文本需显式 `replace_all: true`。同一批次不能重复涉及同一路径。

跨仓库依赖可增加 `--related-owner` 与 `--related-repo`；`blocked_by` 表示当前 issue 被关联 issue 阻塞，`blocking` 表示当前 issue 阻塞关联 issue。

项目命令默认操作当前用户项目，可指定 `--org ORG`，或成对指定 `--owner OWNER --repo REPO`；三种范围互斥：

```bash
gitea-cli projects list --org ORG --state open
gitea-cli projects create --org ORG --title "Delivery" --template-type basic_kanban
gitea-cli projects get --org ORG --project-id 1
gitea-cli projects update --org ORG --project-id 1 --state closed
gitea-cli projects columns --org ORG --project-id 1
gitea-cli projects column-create --org ORG --project-id 1 --title Todo
gitea-cli projects column-update --org ORG --project-id 1 --column-id 2 --title Done
gitea-cli projects column-issues --org ORG --project-id 1 --column-id 2
gitea-cli projects issue-add --org ORG --project-id 1 --issue-id 123 --column-id 2
gitea-cli projects issue-remove --org ORG --project-id 1 --issue-id 123 --yes
gitea-cli projects column-delete --org ORG --project-id 1 --column-id 2 --yes
gitea-cli projects delete --org ORG --project-id 1 --yes
```

`--issue-id` 是 issue 全局 ID，不是仓库内编号；`issue-add` 也可将项目内已有 issue 移动到指定列。

### 连接配置透传

在已有 profile 的 `[profiles.default.mcp.env]` 下按需添加：

```toml
GITEA_CLIENT_CERT = "/path/to/client-cert.pem"
GITEA_CLIENT_KEY = "/path/to/client-key.pem"
GITEA_CA_CERT = "/path/to/ca-bundle.pem"
GITEA_EXTRA_HEADERS = '{"CF-Access-Client-Id":"YOUR_CLIENT_ID","CF-Access-Client-Secret":"YOUR_CLIENT_SECRET"}'
# GITEA_UNIX_SOCKET = "/path/to/gitea.sock"
```

也可通过 `mcp.args` 传入 `-client-cert`、`-client-key`、`-ca-cert`、`-gitea-unix-socket`；证书与私钥需成对配置。额外请求头支持 Cloudflare Access 等认证代理。`config show` / `doctor` 只展示环境变量名称，不展示这些变量值。Homebrew 1.9.0 包的服务端可能报告 `dev`，请同时核对安装版本及实际工具契约。

## Gitea MCP 1.7.0 Compatibility

`gitea-cli 1.7.0` 对齐官方 Gitea MCP Server 1.7.0：

- 新增 `repos branch-rename`，封装官方新增的 `rename_branch` 工具
- `repos file-update --new-branch` 会受益于上游修复，更新已有文件时会正确在指定新分支提交
- 已验证 1.7.0 从 `mcp-go` 切换到官方 MCP Go SDK 后，现有 stdio 初始化、工具发现和 JSON Lines 消息仍兼容
- 1.7.0 新增的 HTTP `--bind`、`/healthz` 与 MCP 2026-07-28 协议支持属于服务端 HTTP 部署能力；`gitea-cli` 继续通过 stdio 启动并调用服务端
- `pulls review-comments` 的 `--review-id` 改为可选，不传时可一次读取整个 Pull Request 的 review comments
- 新增 `pulls review-comment-reply`，回复单条 review comment
- 新增 `pulls review-thread-resolve/review-thread-unresolve`，控制 review thread 的 resolved 状态
- 新增的 `review_id`、`resolved_by` 结果字段会通过稳定 JSON 输出透明返回
- 新增 `issues attachments list/get/download`，覆盖 issue 与评论附件的元数据读取和下载
- 新增 `actions job`，读取单个 Actions job 的完整详情
- 配置中的 `profiles.<name>.mcp.args` 可直接透传 MCP 1.5.0 的 `-S/--scope` 参数；scope 与 `--tools` 按并集合并
- 保留对非文本 MCP content 的透明 JSON 输出，附件内联图片或下载结果不会被错误归一化

Review comment 回复需要目标 Gitea 实例为 1.27.0+；thread resolve/unresolve 需要 Gitea 1.26.0+。版本较低的实例仍可使用其他命令，但调用这些新 method 时会由服务端返回不支持错误。

## Requirements

- Rust toolchain
- 可执行的 Gitea MCP Server，例如官方项目 [`gitea/gitea-mcp`](https://gitea.com/gitea/gitea-mcp)
- 已配置好的 `~/.config/gitea-cli/config.toml`

示例配置：

```toml
version = 1
active_profile = "default"

[profiles.default]
host = "https://your-gitea.example.com"

[profiles.default.mcp]
command = "gitea-mcp-server"
args = ["-t", "stdio"]

[profiles.default.mcp.env]

[profiles.default.auth]
type = "token_env"
token_env = "GITEA_ACCESS_TOKEN"
```

认证也可以使用 `token_file` 或 `token`，其中明文 `token` 会在 `doctor` 中提示安全风险。

如果只希望启动部分 MCP 工具域，可在 Gitea MCP 1.5.0+ 中配置：

```toml
args = ["-t", "stdio", "--scope", "issue,actions"]
```

scope 会影响可调用工具；使用上面的示例时，其他高层命令会因为对应 MCP 工具未加载而返回明确错误。

## Installation

推荐通过 Homebrew 安装：

```bash
brew tap fanbuz/tap
brew install fanbuz/tap/gitea-cli
```

在已支持的平台上，Homebrew 会直接安装 GitHub Release 里的预编译二进制，不需要本地 Rust 编译环境。
当前预编译覆盖：`macOS arm64`、`macOS amd64`、`Linux x64`、`Windows x64`。
当目标平台暂时没有对应预编译包时，才会回退到源码构建。
每次发布新的正式版本后，`fanbuz/homebrew-tap` 会定期检查最新 GitHub Release 并更新 `gitea-cli` formula，无需在本仓库保存跨仓库 token。

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
gitea-cli --json mcp call issue_read --params '{"owner":"YOUR_ORG","repo":"YOUR_REPO","issue_number":123,"method":"get"}'
```

## Command Surface

全局参数补充：

- `--json`
  输出 JSON，适合与 agent、脚本、`jq` 等组合使用。

- `--fields`
  仅在 JSON 输出中保留指定字段，支持逗号分隔和点号路径，例如 `kind,result.parsed.id,result.parsed.title`。

### Config

- `gitea-cli config path`
  输出当前将要使用的配置文件路径。

- `gitea-cli config init --profile default --host https://your-gitea.example.com`
  生成最小可编辑配置文件。

- `gitea-cli config show --json`
  展示脱敏后的配置。

- `gitea-cli config validate --json`
  校验配置结构、MCP 命令和认证来源。

### Health

- `gitea-cli --json doctor`
  检查 `~/.config/gitea-cli/config.toml`、MCP 命令是否在 `PATH` 中、MCP 能否启动、认证是否可用，以及当前可用工具数量。

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

- `gitea-cli --json repos branch-rename --owner YOUR_ORG --repo YOUR_REPO --branch feature/old-name --new-name feature/new-name`
  重命名指定仓库分支，对齐 Gitea MCP 1.7.0 新增的 `rename_branch` 工具。

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

- `gitea-cli --json issues attachments list --owner YOUR_ORG --repo YOUR_REPO --index 123`
  列出 issue 附件；改用 `--comment-id 88` 可列出指定评论的附件。

- `gitea-cli --json issues attachments get --owner YOUR_ORG --repo YOUR_REPO --index 123 --attachment-id 7`
  读取单个 issue 或评论附件的元数据。

- `gitea-cli --json issues attachments download --owner YOUR_ORG --repo YOUR_REPO --attachment-uuid UUID --output-path ./attachment.png`
  按 UUID 下载附件到指定路径；也可用 `--attachment-id` 配合 `--index` 或 `--comment-id` 定位附件。

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

- `gitea-cli --json pulls review-comments --owner YOUR_ORG --repo YOUR_REPO --index 12`
  一次读取整个 Pull Request 的 review comments；可选传入 `--review-id 7` 只读取指定 review。

- `gitea-cli --json pulls review-comments --owner YOUR_ORG --repo YOUR_REPO --index 12 --comment-id 101`
  读取某个 Pull Request review 下的评论列表，并在本地仅保留指定评论。

- `gitea-cli --json pulls review-comment-reply --owner YOUR_ORG --repo YOUR_REPO --index 12 --comment-id 101 --body "fixed"`
  回复单条 Pull Request review comment，需要 Gitea 1.27.0+。

- `gitea-cli --json pulls review-thread-resolve --owner YOUR_ORG --repo YOUR_REPO --comment-id 101`
  将 comment 所在 review thread 标记为 resolved，需要传入线程第一条评论的 ID，并要求 Gitea 1.26.0+。

- `gitea-cli --json pulls review-thread-unresolve --owner YOUR_ORG --repo YOUR_REPO --comment-id 101`
  将已 resolved 的 review thread 恢复为 unresolved。

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

- `gitea-cli --json actions job --owner YOUR_ORG --repo YOUR_REPO --job-id 789`
  读取单个 Actions job 的完整详情。

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
  已提供 `repos branch-create`、`repos branch-rename`、`repos branch-delete`。

- [x] Release / Tag / Commit
  已提供 `releases list/latest/get/create/delete`、`tags list/get/create/delete`、`commits list/get`。

- [x] 文件与目录内容管理
  已提供 `repos dir`、`repos file`、`repos file-create`、`repos file-update`、`repos file-delete`。

- [x] Issue 读取与写操作
  已提供 `issues list/get/comments/search/create/update/comment-add/comment-edit/labels/labels-add/label-remove/labels-replace/labels-clear`，其中 `issues comments` 支持通过可重复 `--comment-id` 过滤评论子集。

- [x] Issue 与评论附件
  已提供 `issues attachments list/get/download`。

- [x] Pull Request 主体读写操作
  已提供 `pulls list/create/update/merge/get/diff/review-comments/reviews/review-get`。

- [x] Pull Request 评审与 reviewer 管理
  已提供 `pulls reviewers-add/reviewers-remove/review-create/review-submit/review-delete/review-dismiss/review-comment-reply/review-thread-resolve/review-thread-unresolve`。

- [x] Actions 只读排查路径
  已提供 `actions workflows`、`actions runs`、`actions jobs`、`actions job`、`actions log-preview`。

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

- `fanbuz/homebrew-tap` 会定期检查最新 GitHub Release，并自行更新 `gitea-cli` formula；
  release workflow 不再依赖跨仓库 token，因此发布不会因为 Tap 凭据失效而失败。

- 如果某个历史标签早于 workflow 提交，比如已经存在的 `v0.0.1`
  可以在 GitHub Actions 页面手动触发 `Release Binaries`，并把 `tag` 输入设为对应版本号，补发 release 产物。

- 当前产物格式：
  Linux / macOS 为 `tar.gz`
  Windows 为 `zip`

## Roadmap

- 补充 wiki、PR 写操作、Actions 写操作等高层命令
- 增加更完整的分页和过滤支持
- 在未来引入写命令前补充更清晰的安全护栏
- 支持项目目录级配置覆盖、repo alias 与 URL alias

## Contributing

欢迎提交 Issue 和 Pull Request。开始之前请先阅读 [CONTRIBUTING.md](CONTRIBUTING.md)。

## Security

安全问题请参考 [SECURITY.md](SECURITY.md)。

## License

本项目使用 [MIT License](LICENSE)。
