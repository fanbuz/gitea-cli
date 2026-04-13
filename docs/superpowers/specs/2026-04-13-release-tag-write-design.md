# gitea-cli Release And Tag Write Command Design

Date: 2026-04-13
Status: Ready for implementation
Target issue: #7
Target milestone: 0.0.7

## Goal

在不调整底层 MCP 会话层和 JSON 输出契约的前提下，为 `gitea-cli` 补齐 release 与 tag 的常见写操作高层命令，覆盖创建与删除两条主路径。

本轮目标是让 agent、脚本和终端用户在版本管理场景下优先使用稳定、带帮助说明的高层 CLI，而不是直接回退到 `gitea-cli --json mcp call ...`。

## Scope

In:

- `releases create`
- `releases delete`
- `tags create`
- `tags delete`
- `src/cli.rs` 中的 clap 参数定义与命令映射
- `tests/command_plans.rs` 中的映射与 help 断言
- README 中 Releases / Tags 段落与 coverage checklist

Out:

- release workflow 触发、产物构建、上传与 Homebrew 通知
- 版本号规划、里程碑流转与发版流程编排
- release / tag 读取命令之外的输出结构调整
- 直接扩展 `mcp call` 行为或底层 MCP 抽象

## Design Principles

- 延续现有命令组织方式，把写操作继续挂在 `releases` 与 `tags` 域下，不额外引入新的顶级命令域。
- 只暴露底层 MCP 当前真实支持且 issue 明确要求的字段，不额外设计新的包装语义。
- 删除命令统一沿用现有危险操作护栏，必须显式传 `--yes`。
- 可选参数未显式传入时，不写入底层请求体，避免高层默认值污染真实请求。
- `releases create` 与 `tags create` 的参数命名尽量统一，便于记忆与脚本复用。

## Command Surface

### Release Create

```bash
gitea-cli --json releases create \
  --owner YOUR_ORG \
  --repo YOUR_REPO \
  --tag v0.0.7 \
  --title "v0.0.7" \
  --target main \
  [--body "..."] \
  [--draft] \
  [--pre-release]
```

对应底层 `create_release`，本轮暴露：

- `owner`
- `repo`
- `tag`
- `title`
- `target`
- `body`
- `draft`
- `pre-release`

其中：

- `--tag` 映射到底层 `tag_name`
- `--target` 映射到底层 `target`
- `--pre-release` 映射到底层 `is_pre_release`
- `--target` 设为必填，避免高层命令对发布基点做隐式推断

### Release Delete

```bash
gitea-cli --json releases delete \
  --owner YOUR_ORG \
  --repo YOUR_REPO \
  --id 12 \
  --yes
```

对应底层 `delete_release`，本轮暴露：

- `owner`
- `repo`
- `id`
- `yes`

### Tag Create

```bash
gitea-cli --json tags create \
  --owner YOUR_ORG \
  --repo YOUR_REPO \
  --tag v0.0.7 \
  [--target main] \
  [--message "annotated tag"]
```

对应底层 `create_tag`，本轮暴露：

- `owner`
- `repo`
- `tag`
- `target`
- `message`

其中：

- `--tag` 映射到底层 `tag_name`
- `--target` 可选，不传时不向底层写入该字段
- `--message` 可选，用于 annotated tag

### Tag Delete

```bash
gitea-cli --json tags delete \
  --owner YOUR_ORG \
  --repo YOUR_REPO \
  --tag v0.0.7 \
  --yes
```

对应底层 `delete_tag`，本轮暴露：

- `owner`
- `repo`
- `tag`
- `yes`

## MCP Mapping

### `releases create`

```json
{
  "owner": "...",
  "repo": "...",
  "tag_name": "v0.0.7",
  "title": "v0.0.7",
  "target": "main",
  "body": "...",
  "is_draft": true,
  "is_pre_release": true
}
```

### `releases delete`

```json
{
  "owner": "...",
  "repo": "...",
  "id": 12
}
```

### `tags create`

```json
{
  "owner": "...",
  "repo": "...",
  "tag_name": "v0.0.7",
  "target": "main",
  "message": "annotated tag"
}
```

### `tags delete`

```json
{
  "owner": "...",
  "repo": "...",
  "tag_name": "v0.0.7"
}
```

## Validation And Error Handling

- `releases create` 的 `owner`、`repo`、`tag`、`title`、`target` 必填。
- `releases delete` 的 `owner`、`repo`、`id` 必填，并且必须显式传 `--yes`。
- `tags create` 的 `owner`、`repo`、`tag` 必填；`target` 与 `message` 可选。
- `tags delete` 的 `owner`、`repo`、`tag` 必填，并且必须显式传 `--yes`。
- 对布尔开关仅在显式传入时写入底层请求体。
- README 和 help 只描述当前真实落地的 release/tag 写操作，不扩展到发版自动化。

## Files And Responsibilities

- `src/cli.rs`
  扩展 `ReleasesSubcommand` 与 `TagsSubcommand`，增加 create/delete 参数结构体，并在 `plan_releases(...)`、`plan_tags(...)` 中完成到底层 MCP tool 的映射。

- `tests/command_plans.rs`
  增加 release/tag 写操作映射测试、危险操作护栏测试，以及 help 文案断言。

- `README.md`
  补齐 release/tag 写操作示例与说明，并把相应 coverage checklist 改为已实现。

## Testing Strategy

- 先为 `releases create` 写失败测试，再补最小实现。
- 再为 `releases delete`、`tags create`、`tags delete` 增加失败测试与 `--yes` 护栏校验。
- 为 `releases --help`、`tags --help` 增加子命令断言，确保 CLI 暴露面与 README 同步。
- 最终执行针对性 `cargo test --test command_plans releases_`、`cargo test --test command_plans tags_` 与全量 `cargo test`。
