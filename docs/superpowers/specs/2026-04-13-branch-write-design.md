# gitea-cli Branch Write Command Design

Date: 2026-04-13
Status: Ready for implementation
Target issue: #5
Target milestone: 0.0.7

## Goal

在不改动底层 MCP 通信层和输出契约的前提下，为 `gitea-cli` 补齐分支管理高层命令，覆盖创建与删除两条常见写路径。

本轮目标是让 agent 和脚本在仓库分支管理场景下优先使用稳定、可记忆、带帮助说明的高层 CLI，而不是退回到 `gitea-cli --json mcp call ...`。

## Scope

In:

- `repos branch-create`
- `repos branch-delete`
- `src/cli.rs` 中的 clap 参数定义与 `plan_repos(...)` 映射
- `tests/command_plans.rs` 中的命令映射与 help 断言
- README Repos 段落与 coverage checklist

Out:

- 仓库创建、fork、搜索等高级仓库管理
- 分支保护规则与高级分支策略配置
- Pull Request、文件内容管理与提交写入
- 底层 MCP 会话层与输出格式调整

## Design Principles

- 延续现有命令风格，把分支写操作继续放在 `repos` 域下，而不是额外引入新的顶级命令域。
- 保留当前 `repos branches` 只读命令不变，新增动词型子命令承接写操作。
- 只暴露底层 `create_branch` 与 `delete_branch` 当前真实支持的字段，不为高层 CLI 设计额外抽象层。
- 删除分支属于危险操作，必须沿用现有 `--yes` 护栏策略。
- 可选参数未显式传入时，不写入底层请求体。

## Command Surface

### Create

```bash
gitea-cli --json repos branch-create \
  --owner YOUR_ORG \
  --repo YOUR_REPO \
  --branch feature/new-command \
  --from main
```

Create 对应底层 `create_branch`，本轮暴露：

- `owner`
- `repo`
- `branch`
- `from`

其中：

- `--branch` 表示要创建的新分支名
- `--from` 映射到底层 `old_branch`
- `--from` 设为必填，避免高层命令在“从哪里切”这件事上产生隐式默认

### Delete

```bash
gitea-cli --json repos branch-delete \
  --owner YOUR_ORG \
  --repo YOUR_REPO \
  --branch feature/new-command \
  --yes
```

Delete 对应底层 `delete_branch`，本轮暴露：

- `owner`
- `repo`
- `branch`
- `yes`

其中：

- `--branch` 表示要删除的分支名
- 未传 `--yes` 时直接报错，不调用底层 MCP

## MCP Mapping

### `repos branch-create`

```json
{
  "owner": "...",
  "repo": "...",
  "branch": "feature/new-command",
  "old_branch": "main"
}
```

### `repos branch-delete`

```json
{
  "owner": "...",
  "repo": "...",
  "branch": "feature/new-command"
}
```

## Validation And Error Handling

- `branch-create` 的 `owner`、`repo`、`branch`、`from` 必填。
- `branch-delete` 的 `owner`、`repo`、`branch` 必填。
- `branch-delete` 必须显式传入 `--yes`，否则报错并阻止执行。
- `branch-create` 不额外引入默认分支推断或交互确认，保持高层命令可脚本化。
- README 和 help 只描述当前真实可用的分支写能力。

## Files And Responsibilities

- `src/cli.rs`
  扩展 `ReposSubcommand`，新增分支创建/删除参数结构体，并在 `plan_repos(...)` 中完成到 `create_branch` 与 `delete_branch` 的映射。

- `tests/command_plans.rs`
  增加 `branch-create`、`branch-delete` 的命令映射测试，以及 Repos help 文案断言。

- `README.md`
  补齐 Repos 分支写操作示例，并将 coverage checklist 中的分支写操作改为已实现。

## Testing Strategy

- 先为 `branch-create`、`branch-delete` 写命令映射测试，确认红灯后再补实现。
- 为 `branch-delete` 覆盖未传 `--yes` 的失败路径。
- 为 `repos --help` 或 `repos` 子命令 help 补一条写操作断言，确保文档面与 CLI 实际暴露一致。
- 最终执行针对性 `cargo test --test command_plans repos_` 与全量 `cargo test`。
