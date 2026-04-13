# gitea-cli Actions Write Command Design

Date: 2026-04-13
Status: Ready for implementation
Target issue: #4
Target milestone: 0.0.7

## Goal

在不改动底层 MCP 通信层和输出契约的前提下，为 `gitea-cli` 补齐 Actions 执行控制高层命令，覆盖 workflow dispatch、run cancel、run rerun 三条常见写路径。

本轮目标是让 agent 和脚本在 Actions 控制场景下优先使用稳定、可记忆、help 友好的高层 CLI，而不是频繁退回到 `gitea-cli --json mcp call ...`。

## Scope

In:

- `actions dispatch`
- `actions cancel`
- `actions rerun`
- `src/cli.rs` 中的 clap 参数定义与 `plan_actions(...)` 映射
- `tests/command_plans.rs` 中的命令映射与 help 断言
- README Actions 段落与 coverage checklist

Out:

- Actions secrets 与 variables 管理
- workflow 文件或配置管理
- 新的日志下载或只读排查命令
- 底层 MCP 会话层与输出格式调整

## Design Principles

- 延续现有命令风格：资源名在前，动词型子命令在后。
- 只暴露底层 `actions_run_write` 当前真实支持的字段，不为高层 CLI 设计无法稳定映射的抽象层。
- 可选参数未显式传入时，不写入底层请求体，避免把空值误传给 MCP。
- `dispatch` 的 `--inputs` 继续沿用现有 JSON 参数解析模型，同时支持内联 JSON 与 `@file` 方式。
- `cancel` 与 `rerun` 保持轻量命令面，不额外加二次确认参数，确保可脚本化。

## Command Surface

### Dispatch

```bash
gitea-cli --json actions dispatch \
  --owner YOUR_ORG \
  --repo YOUR_REPO \
  --workflow-id release.yml \
  --ref main \
  [--inputs '{"env":"prod"}'] \
  [--inputs @inputs.json]
```

Dispatch 对应底层 `actions_run_write method=dispatch_workflow`，本轮暴露：

- `owner`
- `repo`
- `workflow_id`
- `ref`
- `inputs`

其中：

- `workflow_id` 直接传递到底层，不额外区分 workflow 名称或文件名来源
- `ref` 必填，保持高层命令语义明确
- `inputs` 为可选 JSON 对象，支持：
  - `--inputs '{"env":"prod"}'`
  - `--inputs @inputs.json`

### Cancel

```bash
gitea-cli --json actions cancel \
  --owner YOUR_ORG \
  --repo YOUR_REPO \
  --run-id 456
```

Cancel 对应底层 `actions_run_write method=cancel_run`，本轮只暴露：

- `owner`
- `repo`
- `run_id`

### Rerun

```bash
gitea-cli --json actions rerun \
  --owner YOUR_ORG \
  --repo YOUR_REPO \
  --run-id 456
```

Rerun 对应底层 `actions_run_write method=rerun_run`，本轮只暴露：

- `owner`
- `repo`
- `run_id`

## MCP Mapping

### `actions dispatch`

```json
{
  "method": "dispatch_workflow",
  "owner": "...",
  "repo": "...",
  "workflow_id": "release.yml",
  "ref": "main",
  "inputs": {
    "env": "prod"
  }
}
```

### `actions cancel`

```json
{
  "method": "cancel_run",
  "owner": "...",
  "repo": "...",
  "run_id": 456
}
```

### `actions rerun`

```json
{
  "method": "rerun_run",
  "owner": "...",
  "repo": "...",
  "run_id": 456
}
```

## Validation And Error Handling

- `dispatch` 的 `owner`、`repo`、`workflow_id`、`ref` 必填。
- `cancel` 和 `rerun` 的 `owner`、`repo`、`run_id` 必填。
- `--inputs` 如果存在：
  - 以 `@` 开头时，从文件读取 JSON 文本
  - 否则直接按内联 JSON 解析
- `--inputs` 必须解析为 JSON 对象；若是数组、字符串、数字或 `null`，直接报错，避免把非对象负载传给 workflow inputs。
- 所有可选字段都遵循“未传即省略”的请求体构造策略。

## Files And Responsibilities

- `src/cli.rs`
  扩展 `ActionsSubcommand`，新增参数结构体，并在 `plan_actions(...)` 中完成三条写操作到 `actions_run_write` 的映射。

- `tests/command_plans.rs`
  增加 `dispatch`、`cancel`、`rerun` 的命令映射测试，以及 Actions help 文案断言。

- `README.md`
  补齐 Actions 写操作示例，并将 coverage checklist 调整为已覆盖执行控制高层命令。

## Testing Strategy

- 先为 `dispatch`、`cancel`、`rerun` 写命令映射测试，确认红灯后再补实现。
- 为 `dispatch --inputs` 覆盖内联 JSON 对象解析。
- 为 help 输出补一条 Actions 子命令描述断言，确保文档面与 CLI 实际暴露一致。
- 最终执行针对性 `cargo test --test command_plans actions_` 与全量 `cargo test`。
