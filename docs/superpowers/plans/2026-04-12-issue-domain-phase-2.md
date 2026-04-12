# Issue Domain Phase 2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 `gitea-cli` 补齐 issue domain 第二阶段高层命令，覆盖 repo/org labels 与 milestones 的读取、写入和危险操作护栏。

**Architecture:** 延续现有 `src/cli.rs` 的 clap 子命令与 `plan_command` 映射模式，新增 `labels` 和 `milestones` 顶层命令，不改动 MCP 通信层。所有危险删除命令都通过 `require_yes(...)` 统一拦截，并在 `tests/command_plans.rs` 中补足命令映射与 help 文案测试。

**Tech Stack:** Rust 2024、clap 4、serde_json、cargo test

---

### Task 1: 为 `labels` 顶层命令写失败测试

**Files:**
- Modify: `tests/command_plans.rs`
- Test: `tests/command_plans.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn labels_repo_create_maps_to_label_write_create_repo_label() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "labels",
        "repo-create",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--name",
        "bug",
        "--color",
        "#ff0000",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "label_write",
            serde_json::json!({
                "method": "create_repo_label",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "name": "bug",
                "color": "#ff0000"
            })
        )
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --locked --test command_plans labels_`
Expected: FAIL，提示 `labels` 顶层命令尚未定义。

- [ ] **Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, Args)]
pub struct LabelsCommand {
    #[command(subcommand)]
    pub command: LabelsSubcommand,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --locked --test command_plans labels_`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/command_plans.rs src/cli.rs
git commit -m "feat: add label commands"
```

### Task 2: 为 `milestones` 顶层命令写失败测试

**Files:**
- Modify: `tests/command_plans.rs`
- Modify: `src/cli.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn milestones_create_maps_to_milestone_write_create() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "milestones",
        "create",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--title",
        "v0.0.6",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "milestone_write",
            serde_json::json!({
                "method": "create",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "title": "v0.0.6"
            })
        )
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --locked --test command_plans milestones_`
Expected: FAIL，提示 `milestones` 顶层命令尚未定义。

- [ ] **Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, Args)]
pub struct MilestonesCommand {
    #[command(subcommand)]
    pub command: MilestonesSubcommand,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --locked --test command_plans milestones_`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/command_plans.rs src/cli.rs
git commit -m "feat: add milestone commands"
```

### Task 3: 为删除类命令补 `--yes` 护栏与 help 断言

**Files:**
- Modify: `tests/command_plans.rs`
- Modify: `src/cli.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn labels_repo_delete_requires_yes() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "labels",
        "repo-delete",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--id",
        "9",
    ])
    .unwrap();

    let error = plan_command(&cli).unwrap_err();

    assert!(error.to_string().contains("--yes"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --locked --test command_plans labels_repo_delete_requires_yes milestones_delete_requires_yes`
Expected: FAIL，说明删除命令还没有统一护栏。

- [ ] **Step 3: Write minimal implementation**

```rust
require_yes(args.yes, "删除 repo label")?;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --locked --test command_plans labels_repo_delete_requires_yes milestones_delete_requires_yes`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/command_plans.rs src/cli.rs
git commit -m "feat: guard destructive label and milestone commands"
```

### Task 4: 更新 README 与 coverage checklist

**Files:**
- Modify: `README.md`
- Test: `tests/command_plans.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn top_level_help_includes_label_and_milestone_commands() {
    let help = render_help(Cli::command());

    assert!(help.contains("labels"));
    assert!(help.contains("milestones"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --locked --test command_plans top_level_help_includes_label_and_milestone_commands`
Expected: FAIL，顶层 help 尚未暴露新命令。

- [ ] **Step 3: Write minimal implementation**

```md
### Labels
- `gitea-cli --json labels repo-list ...`

### Milestones
- `gitea-cli --json milestones list ...`
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --locked --test command_plans top_level_help_includes_label_and_milestone_commands`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add README.md tests/command_plans.rs src/cli.rs
git commit -m "docs: add label and milestone commands"
```
