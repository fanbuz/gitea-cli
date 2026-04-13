# gitea-cli Pull Request Write Command Design

Date: 2026-04-13
Status: Ready for review
Target issue: #2
Target milestone: 0.0.7

## Goal

在不改动底层 MCP 通信层和输出契约的前提下，为 `gitea-cli` 补齐 Pull Request 主体写操作高层命令，覆盖创建、更新和合并三条主路径。

本轮目标是让用户和 agent 在常见 PR 主体协作流中优先使用稳定、可记忆、带帮助说明的高层 CLI，而不是频繁回退到 `gitea-cli --json mcp call ...`。

## Scope

In:

- `pulls create`
- `pulls update`
- `pulls merge`
- `src/cli.rs` 中的 clap 参数定义与 `plan_pulls(...)` 映射
- `tests/command_plans.rs` 中的命令映射与 help 断言
- README Pull Requests 段落与 coverage checklist

Out:

- reviewer 管理与 review 生命周期命令
- Pull Request 以外的仓库写操作
- MCP 通信层与输出格式调整
- 独立配置层

## Design Principles

- 继续保持现有命令风格：资源名在前，动词型子命令在后。
- 只暴露底层 `pull_request_write` 当前真实支持的字段，不为高层 CLI 设计“理想但不可映射”的参数。
- 可选参数未显式传入时，不写入底层请求体，避免把空值误传给 MCP。
- 与现有 issue domain 命令保持参数风格一致，重复项继续采用 `--assignee`、`--label-id` 这类可重复参数。
- `merge` 不额外引入 `--yes` 二次确认，保持 CLI 可脚本化；风险通过显式子命令和危险参数名体现。

## Command Surface

### Create

```bash
gitea-cli --json pulls create \
  --owner YOUR_ORG \
  --repo YOUR_REPO \
  --head feature-branch \
  --base main \
  --title "Add feature" \
  [--body "details"] \
  [--label-id 1 --label-id 2] \
  [--draft] \
  [--deadline 2026-04-30T12:00:00Z]
```

Create 只暴露底层 `pull_request_write method=create` 当前可稳定映射的字段：

- `owner`
- `repo`
- `head`
- `base`
- `title`
- `body`
- `labels`
- `draft`
- `deadline`

本轮不为 `create` 暴露 `milestone`、`assignee`、`allow_maintainer_edit`，因为它们不属于底层 create 方法当前支持的字段集合。

### Update

```bash
gitea-cli --json pulls update \
  --owner YOUR_ORG \
  --repo YOUR_REPO \
  --index 12 \
  [--title "New title"] \
  [--body "details"] \
  [--state open|closed] \
  [--base release/0.0.7] \
  [--assignee mashu --assignee fanbuz] \
  [--label-id 3 --label-id 4] \
  [--milestone 7] \
  [--deadline 2026-04-30T12:00:00Z] \
  [--remove-deadline] \
  [--allow-maintainer-edit true|false] \
  [--draft true|false]
```

Update 对应底层 `pull_request_write method=update`，允许映射：

- `title`
- `body`
- `state`
- `base`
- `assignee`
- `assignees`
- `labels`
- `milestone`
- `deadline`
- `remove_deadline`
- `allow_maintainer_edit`
- `draft`

其中：

- 本轮只保留可重复 `--assignee` 参数
- 当 `--assignee` 仅传入一个值时，映射到底层 `assignee`
- 当 `--assignee` 传入多个值时，映射到底层 `assignees`
- `remove_deadline` 只在显式传入时写入 `true`
- `allow_maintainer_edit` 与 `draft` 采用显式布尔值，避免 update 场景下“仅传 flag 但无法表达 false”

### Merge

```bash
gitea-cli --json pulls merge \
  --owner YOUR_ORG \
  --repo YOUR_REPO \
  --index 12 \
  [--merge-style merge|rebase|rebase-merge|squash|fast-forward-only] \
  [--title "merge title"] \
  [--message "merge message"] \
  [--delete-branch] \
  [--force-merge] \
  [--merge-when-checks-succeed] \
  [--head-commit-id COMMIT_SHA]
```

Merge 对应底层 `pull_request_write method=merge`，允许映射：

- `merge_style`
- `title`
- `message`
- `delete_branch`
- `force_merge`
- `merge_when_checks_succeed`
- `head_commit_id`

本轮不额外引入二次确认参数。原因是：

- `pulls merge` 本身就是明确写操作
- 现有 CLI 只有明显 destructive 的删除/清空类命令才强制 `--yes`
- 增加额外确认会削弱 agent 与 shell 自动化场景的可脚本性

## MCP Mapping

### `pulls create`

```json
{
  "method": "create",
  "owner": "...",
  "repo": "...",
  "head": "...",
  "base": "...",
  "title": "...",
  "body": "...",
  "labels": [1, 2],
  "draft": true,
  "deadline": "2026-04-30T12:00:00Z"
}
```

### `pulls update`

```json
{
  "method": "update",
  "owner": "...",
  "repo": "...",
  "index": 12,
  "title": "...",
  "body": "...",
  "state": "closed",
  "base": "release/0.0.7",
  "assignee": "fanbuz",
  "labels": [3, 4],
  "milestone": 7,
  "deadline": "2026-04-30T12:00:00Z",
  "remove_deadline": true,
  "allow_maintainer_edit": false,
  "draft": true
}
```

### `pulls merge`

```json
{
  "method": "merge",
  "owner": "...",
  "repo": "...",
  "index": 12,
  "merge_style": "squash",
  "title": "...",
  "message": "...",
  "delete_branch": true,
  "force_merge": true,
  "merge_when_checks_succeed": true,
  "head_commit_id": "abcdef123456"
}
```

## Validation And Error Handling

- `create` 的 `owner`、`repo`、`head`、`base`、`title` 必填。
- `update` 和 `merge` 的 `owner`、`repo`、`index` 必填。
- `merge_style` 通过 clap value enum 或等价受限字符串收口到 MCP 支持集合。
- `allow_maintainer_edit` 与 `draft` 在 update 中必须显式传值，不使用仅存在即为 true 的 flag 语义。
- 所有可选字段都遵循“未传即省略”的请求体构造策略。
- `create` 不暴露底层不支持的字段；README 和 help 只描述当前真实可用能力。

## Files And Responsibilities

- `src/cli.rs`
  扩展 `PullsSubcommand`、新增参数结构体、在 `plan_pulls(...)` 中完成 create/update/merge 到 `pull_request_write` 的映射。

- `tests/command_plans.rs`
  增加 `pulls create`、`pulls update`、`pulls merge` 的映射测试，以及 Pull Requests help 文案断言。

- `README.md`
  更新 Pull Requests 命令面示例和官方 MCP coverage checklist，把 PR 主体能力从“只读”提升为“主体读写已覆盖，review 生命周期另见 #3”。

## Testing Strategy

- 为 `pulls create` 写至少一个完整映射测试，覆盖 `head`、`base`、`title`、`label-id`、`draft`。
- 为 `pulls update` 写至少一个完整映射测试，覆盖 `state`、`assignee`、`label-id`、`milestone`、`allow_maintainer_edit`、`draft`、`remove_deadline`。
- 为 `pulls merge` 写至少一个完整映射测试，覆盖 `merge_style`、`delete_branch`、`force_merge`、`merge_when_checks_succeed`、`head_commit_id`。
- 为 help 文案增加断言，确认 `pulls` 下出现 `create`、`update`、`merge`。
- 全量执行 `cargo test`。

## Risks

- `pull_request_write` 的 create/update/merge 支持字段并不一致，最容易在参数定义时误把 update 字段带入 create。
- update 中的单值 `assignee` 与多值 `assignees` 容易在映射策略上出现歧义，需要在 spec、help 和实现中保持一致。
- `Option<bool>` 型参数如果 help 描述不清晰，会导致用户误以为支持只传 flag 的布尔开关语义。

## Acceptance Criteria

- `gitea-cli --json pulls create`、`pulls update`、`pulls merge` 都有稳定的高层命令。
- 各命令只暴露当前底层 MCP 可真实映射的字段，不出现“help 里有、请求体里不能写”的假能力。
- 未传可选参数时，不向底层请求体写入空字段。
- README Pull Requests 段落、coverage checklist 与实际命令面一致。
- `cargo test` 通过。
