# gitea-cli Issue Domain Alignment Design

Date: 2026-04-12
Status: Approved for implementation
Target version: next minor after 0.0.5

## Goal

在不改动底层 MCP 通信层的前提下，把 `gitea-cli` 的 issue 相关高层命令补齐到可覆盖日常 issue 工作流的程度。这里的“issue 相关”不仅包括 issue 本体和评论，还包括 labels、milestones、time tracking 这三类与 issue 高度耦合的资源。

本轮目标是让用户在大多数 issue 管理场景里优先使用稳定、可记忆、带帮助说明的高层 CLI，而不是频繁回退到 `gitea-cli --json mcp call ...`。

## Scope

In:

- `issues` 的创建、更新、评论、标签管理
- `issues time` 的读取与写入
- `labels` 的 repo/org 级读取与写入
- `milestones` 的读取与写入
- README 的 command surface、coverage checklist、safety 文案
- 命令映射测试、help 文案测试

Out:

- `wiki`
- `repo/file` 管理
- `pull request` 写操作
- `actions` 写操作与 secrets/variables
- 独立配置层

## Design Principles

- 保持现有命令风格：资源名在前，动词型子命令在后。
- 仓库级资源与 issue 级资源分开建模，避免把 `labels`、`milestones` 全塞进 `issues` 下。
- 继续沿用 `PlannedCommand::tool_call(...)` 映射到底层官方 MCP tool，不在本轮引入额外抽象层。
- 写操作默认允许执行，但对明显 destructive 的命令增加 `--yes` 护栏。
- 所有新命令都要补 help 说明和命令映射测试。

## Command Surface

### Issues

- `issues create --owner --repo --title [--body] [--assignee ...] [--label-id ...] [--milestone <id>] [--ref <branch>] [--deadline <iso8601>]`
- `issues update --owner --repo --index [--title] [--body] [--state <open|closed|all>] [--assignee ...] [--label-id ...] [--milestone <id>] [--ref <branch>] [--deadline <iso8601>] [--remove-deadline]`
- `issues comment-add --owner --repo --index --body`
- `issues comment-edit --owner --repo --index --comment-id --body`
- `issues labels --owner --repo --index`
- `issues labels-add --owner --repo --index --label-id ...`
- `issues label-remove --owner --repo --index --label-id --yes`
- `issues labels-replace --owner --repo --index --label-id ...`
- `issues labels-clear --owner --repo --index --yes`

### Issue Time

- `issues time list --owner --repo --index [--page] [--page-size]`
- `issues time start --owner --repo --index`
- `issues time stop --owner --repo --index`
- `issues time reset-stopwatch --owner --repo --index --yes`
- `issues time add --owner --repo --index --seconds <n>`
- `issues time delete --owner --repo --index --id <time-id> --yes`

### Labels

- `labels repo-list --owner --repo [--page] [--page-size]`
- `labels repo-get --owner --repo --id <label-id>`
- `labels repo-create --owner --repo --name --color [--description] [--archived]`
- `labels repo-edit --owner --repo --id <label-id> [--name] [--color] [--description] [--archived]`
- `labels repo-delete --owner --repo --id <label-id> --yes`
- `labels org-list --org [--page] [--page-size]`
- `labels org-create --org --name --color [--description] [--exclusive]`
- `labels org-edit --org --id <label-id> [--name] [--color] [--description] [--exclusive]`
- `labels org-delete --org --id <label-id> --yes`

### Milestones

- `milestones list --owner --repo [--state <open|closed>] [--name <keyword>] [--page] [--page-size]`
- `milestones get --owner --repo --id <milestone-id>`
- `milestones create --owner --repo --title [--description] [--due-on <iso8601>]`
- `milestones edit --owner --repo --id <milestone-id> [--title] [--description] [--due-on <iso8601>] [--state <open|closed>]`
- `milestones delete --owner --repo --id <milestone-id> --yes`

## MCP Mapping

### Issues

- `issues create` -> `issue_write` with `method=create`
- `issues update` -> `issue_write` with `method=update`
- `issues comment-add` -> `issue_write` with `method=add_comment`
- `issues comment-edit` -> `issue_write` with `method=edit_comment`
- `issues labels` -> `issue_read` with `method=get_labels`
- `issues labels-add` -> `issue_write` with `method=add_labels`
- `issues label-remove` -> `issue_write` with `method=remove_label`
- `issues labels-replace` -> `issue_write` with `method=replace_labels`
- `issues labels-clear` -> `issue_write` with `method=clear_labels`

### Time Tracking

- `issues time list` -> `timetracking_read` with `method=list_issue_times`
- `issues time start` -> `timetracking_write` with `method=start_stopwatch`
- `issues time stop` -> `timetracking_write` with `method=stop_stopwatch`
- `issues time reset-stopwatch` -> `timetracking_write` with `method=delete_stopwatch`
- `issues time add` -> `timetracking_write` with `method=add_time`
- `issues time delete` -> `timetracking_write` with `method=delete_time`

### Labels

- `labels repo-list` -> `label_read` with `method=list_repo_labels`
- `labels repo-get` -> `label_read` with `method=get_repo_label`
- `labels org-list` -> `label_read` with `method=list_org_labels`
- `labels repo-create` -> `label_write` with `method=create_repo_label`
- `labels repo-edit` -> `label_write` with `method=edit_repo_label`
- `labels repo-delete` -> `label_write` with `method=delete_repo_label`
- `labels org-create` -> `label_write` with `method=create_org_label`
- `labels org-edit` -> `label_write` with `method=edit_org_label`
- `labels org-delete` -> `label_write` with `method=delete_org_label`

### Milestones

- `milestones list` -> `milestone_read` with `method=list`
- `milestones get` -> `milestone_read` with `method=get`
- `milestones create` -> `milestone_write` with `method=create`
- `milestones edit` -> `milestone_write` with `method=edit`
- `milestones delete` -> `milestone_write` with `method=delete`

## Safety Rules

- 删除、清空、重置类命令必须显式传 `--yes`。
- 非 destructive 写命令不加二次确认，保持 CLI 可脚本化。
- README 需要明确说明：高层命令不再只读，本轮开始引入受控写操作。
- `mcp call` 继续保留为兜底出口，但 README coverage checklist 要同步更新为“issue domain 已高层覆盖”。

## Implementation Plan

### Phase 1: Issues Core

- 扩展 `issues` 子命令定义
- 增加 issue 写操作参数结构体
- 增加 `issues time` 嵌套子命令
- 补齐命令映射测试

### Phase 2: Labels And Milestones

- 新增 `labels` 顶层命令
- 新增 `milestones` 顶层命令
- 补齐命令映射测试与 help 测试

### Phase 3: Docs

- 更新 README command surface
- 更新官方 MCP coverage checklist
- 更新 safety 说明和 roadmap

## Testing Strategy

- 对每个新高层命令至少增加一个 `plan_command` 映射测试。
- 对新增顶层命令和关键危险命令增加 help 文案断言。
- 全量执行 `cargo test --locked`。
- 完成后执行 `cargo fmt --check`。

## Risks

- `issue_write`、`label_write`、`milestone_write` 参数较多，最容易出现字段名与 MCP tool 定义不一致的问题。
- `--yes` 护栏如果设计不统一，会让 destructive 命令的行为变得不一致。
- `issues update` 的 optional 参数较多，要避免无值字段错误写入请求体。

## Acceptance Criteria

- Issue 相关常见工作流可不依赖 `mcp call` 完成：创建、更新、评论、标签维护、时间记录、milestone 管理。
- README 中的 issue domain command surface 与实际 help 输出一致。
- 官方 MCP coverage checklist 对 issue domain 的表述更新为“已实现高层命令”。
- 全部测试通过，且 help 中包含新增命令说明。
