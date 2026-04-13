# Actions Write Commands Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 `gitea-cli` 补齐 Actions 执行控制高层命令，覆盖 workflow dispatch、run cancel 和 rerun。

**Architecture:** 延续现有 `src/cli.rs` 中 clap 参数定义加 `plan_actions(...)` 映射到底层 MCP tool 的模式，不改动 `src/mcp.rs` 与输出层。实现按 TDD 分三批推进：先补 `dispatch` 命令与 `--inputs` 映射，再补 `cancel` / `rerun`，最后收 README 与 help 文案。

**Tech Stack:** Rust 2024、clap 4、serde_json、cargo test

---

### Task 1: 为 `actions dispatch` 补失败测试与最小实现

**Files:**
- Modify: `tests/command_plans.rs`
- Modify: `src/cli.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn actions_dispatch_maps_to_actions_run_write_dispatch_workflow() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "actions",
        "dispatch",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--workflow-id",
        "release.yml",
        "--ref",
        "main",
        "--inputs",
        "{\"env\":\"prod\"}",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "actions_run_write",
            serde_json::json!({
                "method": "dispatch_workflow",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "workflow_id": "release.yml",
                "ref": "main",
                "inputs": {
                    "env": "prod"
                }
            })
        )
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test command_plans actions_dispatch_maps_to_actions_run_write_dispatch_workflow -- --exact`  
Expected: FAIL，提示 `actions dispatch` 尚未定义或映射不存在。

- [ ] **Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, Subcommand)]
pub enum ActionsSubcommand {
    Workflows(RepoTargetArgs),
    Runs(ActionsRunsArgs),
    Jobs(ActionsJobsArgs),
    LogPreview(ActionsLogPreviewArgs),
    Dispatch(ActionsDispatchArgs),
}

#[derive(Debug, Clone, Args)]
pub struct ActionsDispatchArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    #[arg(long = "workflow-id")]
    pub workflow_id: String,
    #[arg(long = "ref")]
    pub git_ref: String,
    #[arg(long)]
    pub inputs: Option<String>,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test command_plans actions_dispatch_maps_to_actions_run_write_dispatch_workflow -- --exact`  
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/command_plans.rs src/cli.rs
git commit -m "补齐 Actions dispatch 命令映射 (#4)"
```

### Task 2: 为 `actions cancel` 与 `actions rerun` 补失败测试与最小实现

**Files:**
- Modify: `tests/command_plans.rs`
- Modify: `src/cli.rs`

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn actions_cancel_maps_to_actions_run_write_cancel_run() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "actions",
        "cancel",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--run-id",
        "456",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "actions_run_write",
            serde_json::json!({
                "method": "cancel_run",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "run_id": 456
            })
        )
    );
}

#[test]
fn actions_rerun_maps_to_actions_run_write_rerun_run() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "actions",
        "rerun",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--run-id",
        "456",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "actions_run_write",
            serde_json::json!({
                "method": "rerun_run",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "run_id": 456
            })
        )
    );
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test command_plans actions_cancel_maps_to_actions_run_write_cancel_run actions_rerun_maps_to_actions_run_write_rerun_run`  
Expected: FAIL，提示 `actions cancel` / `actions rerun` 尚未定义或映射不存在。

- [ ] **Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, Subcommand)]
pub enum ActionsSubcommand {
    Workflows(RepoTargetArgs),
    Runs(ActionsRunsArgs),
    Jobs(ActionsJobsArgs),
    LogPreview(ActionsLogPreviewArgs),
    Dispatch(ActionsDispatchArgs),
    Cancel(ActionsRunTargetArgs),
    Rerun(ActionsRunTargetArgs),
}

#[derive(Debug, Clone, Args)]
pub struct ActionsRunTargetArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    #[arg(long = "run-id")]
    pub run_id: u64,
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test command_plans actions_cancel_maps_to_actions_run_write_cancel_run actions_rerun_maps_to_actions_run_write_rerun_run`  
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/command_plans.rs src/cli.rs
git commit -m "补齐 Actions cancel 与 rerun 命令映射 (#4)"
```

### Task 3: 为 Actions help 和 README 收失败测试与文档更新

**Files:**
- Modify: `tests/command_plans.rs`
- Modify: `README.md`

- [ ] **Step 1: Write the failing help test**

```rust
#[test]
fn actions_help_includes_write_subcommand_descriptions() {
    let mut root = Cli::command();
    let actions_help = render_help(find_subcommand(&mut root, "actions").clone());

    assert!(help_has_command_description(
        &actions_help,
        "dispatch",
        "触发 workflow 运行"
    ));
    assert!(help_has_command_description(
        &actions_help,
        "cancel",
        "取消指定 run"
    ));
    assert!(help_has_command_description(
        &actions_help,
        "rerun",
        "重跑指定 run"
    ));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test command_plans actions_help_includes_write_subcommand_descriptions -- --exact`  
Expected: FAIL，help 中还没有三条写操作子命令。

- [ ] **Step 3: Update help 文案与 README**

```md
- `gitea-cli --json actions dispatch --owner YOUR_ORG --repo YOUR_REPO --workflow-id release.yml --ref main --inputs '{"env":"prod"}'`
  触发指定 workflow 运行，支持以内联 JSON 或 `@file` 方式传入 inputs。

- `gitea-cli --json actions cancel --owner YOUR_ORG --repo YOUR_REPO --run-id 456`
  取消指定 workflow run。

- `gitea-cli --json actions rerun --owner YOUR_ORG --repo YOUR_REPO --run-id 456`
  重跑指定 workflow run。
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test command_plans actions_help_includes_write_subcommand_descriptions -- --exact`  
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/command_plans.rs README.md
git commit -m "补齐 Actions 命令文档与帮助信息 (#4)"
```

### Task 4: 完整验证并提交收口

**Files:**
- Verify only: `src/cli.rs`
- Verify only: `tests/command_plans.rs`
- Verify only: `README.md`

- [ ] **Step 1: Run targeted Actions command plan tests**

Run: `cargo test --test command_plans actions_`  
Expected: PASS，包含只读和新增写操作相关测试。

- [ ] **Step 2: Run full test suite**

Run: `cargo test`  
Expected: PASS，无失败测试。

- [ ] **Step 3: Review diff before commit handoff**

Run: `git diff -- src/cli.rs tests/command_plans.rs README.md docs/superpowers/specs/2026-04-13-actions-write-design.md docs/superpowers/plans/2026-04-13-actions-write-implementation.md`  
Expected: 仅包含 `#4` 相关命令、文档、测试与设计计划文件。

- [ ] **Step 4: Commit docs and implementation**

```bash
git add docs/superpowers/specs/2026-04-13-actions-write-design.md docs/superpowers/plans/2026-04-13-actions-write-implementation.md src/cli.rs tests/command_plans.rs README.md
git commit -m "补齐 Actions 执行控制高层命令 (#4)"
```
