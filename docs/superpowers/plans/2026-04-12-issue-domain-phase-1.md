# Issue Domain Phase 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 `gitea-cli` 补齐 issue domain 的第一阶段高层命令，覆盖 issue 创建、更新、评论、标签维护与时间跟踪。

**Architecture:** 保持现有 `src/cli.rs` 的 clap 解析与 `plan_command` 映射模式，不改底层 MCP 通信层。新增命令只扩展参数结构体、子命令枚举和 `PlannedCommand::tool_call(...)` 的参数映射，同时在 `tests/command_plans.rs` 中通过 TDD 补足命令映射与 help 文案断言。

**Tech Stack:** Rust 2024、clap 4、serde_json、cargo test

---

### Task 1: 为 issue 写操作补测试骨架

**Files:**
- Modify: `tests/command_plans.rs`
- Test: `tests/command_plans.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn issues_create_maps_to_issue_write_create() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "issues",
        "create",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--title",
        "need fix",
        "--body",
        "detail",
        "--assignee",
        "mashu",
        "--label-id",
        "3",
        "--milestone",
        "7",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "issue_write",
            serde_json::json!({
                "method": "create",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "title": "need fix",
                "body": "detail",
                "assignees": ["mashu"],
                "labels": [3],
                "milestone": 7
            })
        )
    );
}

#[test]
fn issues_update_maps_to_issue_write_update() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "issues",
        "update",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--index",
        "524",
        "--title",
        "new title",
        "--state",
        "closed",
        "--remove-deadline",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "issue_write",
            serde_json::json!({
                "method": "update",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "index": 524,
                "title": "new title",
                "state": "closed",
                "remove_deadline": true
            })
        )
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --locked issues_create_maps_to_issue_write_create issues_update_maps_to_issue_write_update`
Expected: FAIL，提示 `issues create/update` 子命令尚未定义或映射不存在。

- [ ] **Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, Subcommand)]
pub enum IssuesSubcommand {
    List(IssuesListArgs),
    Get(IssueTargetArgs),
    Comments(IssueTargetArgs),
    Search(IssueSearchArgs),
    Create(IssueCreateArgs),
    Update(IssueUpdateArgs),
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --locked issues_create_maps_to_issue_write_create issues_update_maps_to_issue_write_update`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/command_plans.rs src/cli.rs
git commit -m "feat: add issue create and update commands"
```

### Task 2: 为 issue 评论与标签维护补测试和实现

**Files:**
- Modify: `tests/command_plans.rs`
- Modify: `src/cli.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn issues_comment_add_maps_to_issue_write_add_comment() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "issues",
        "comment-add",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--index",
        "524",
        "--body",
        "follow up",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "issue_write",
            serde_json::json!({
                "method": "add_comment",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "index": 524,
                "body": "follow up"
            })
        )
    );
}

#[test]
fn issues_labels_add_maps_to_issue_write_add_labels() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "issues",
        "labels-add",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--index",
        "524",
        "--label-id",
        "1",
        "--label-id",
        "2",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "issue_write",
            serde_json::json!({
                "method": "add_labels",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "index": 524,
                "labels": [1, 2]
            })
        )
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --locked issues_comment_add_maps_to_issue_write_add_comment issues_labels_add_maps_to_issue_write_add_labels`
Expected: FAIL，提示 `comment-add` / `labels-add` 未定义。

- [ ] **Step 3: Write minimal implementation**

```rust
IssuesSubcommand::CommentAdd(args) => Ok(PlannedCommand::tool_call(
    "issue_write",
    json!({
        "method": "add_comment",
        "owner": args.owner,
        "repo": args.repo,
        "index": args.index,
        "body": args.body
    }),
))
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --locked issues_comment_add_maps_to_issue_write_add_comment issues_labels_add_maps_to_issue_write_add_labels`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/command_plans.rs src/cli.rs
git commit -m "feat: add issue comment and label commands"
```

### Task 3: 为 issue time 子命令补测试和实现

**Files:**
- Modify: `tests/command_plans.rs`
- Modify: `src/cli.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn issues_time_list_maps_to_timetracking_read() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "issues",
        "time",
        "list",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--index",
        "524",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "timetracking_read",
            serde_json::json!({
                "method": "list_issue_times",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "index": 524,
                "page": 1,
                "perPage": 30
            })
        )
    );
}

#[test]
fn issues_time_add_maps_to_timetracking_write_add_time() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "issues",
        "time",
        "add",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--index",
        "524",
        "--seconds",
        "120",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "timetracking_write",
            serde_json::json!({
                "method": "add_time",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "index": 524,
                "time": 120
            })
        )
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --locked issues_time_list_maps_to_timetracking_read issues_time_add_maps_to_timetracking_write_add_time`
Expected: FAIL，提示 `issues time` 子命令缺失。

- [ ] **Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, Args)]
pub struct IssueTimeCommand {
    #[command(subcommand)]
    pub command: IssueTimeSubcommand,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --locked issues_time_list_maps_to_timetracking_read issues_time_add_maps_to_timetracking_write_add_time`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/command_plans.rs src/cli.rs
git commit -m "feat: add issue time commands"
```

### Task 4: 为危险命令补 `--yes` 护栏和 help 断言

**Files:**
- Modify: `tests/command_plans.rs`
- Modify: `src/cli.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn issues_help_includes_destructive_confirmation_flags() {
    let help = render_help(find_subcommand(
        find_subcommand(&mut Cli::command(), "issues"),
        "label-remove",
    ).clone());

    assert!(help.contains("--yes"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --locked issues_help_includes_destructive_confirmation_flags`
Expected: FAIL，help 中没有 `--yes`。

- [ ] **Step 3: Write minimal implementation**

```rust
#[arg(long, help = "确认执行危险操作")]
pub yes: bool,
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --locked issues_help_includes_destructive_confirmation_flags`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/command_plans.rs src/cli.rs
git commit -m "feat: add confirmation flags for destructive issue commands"
```

### Task 5: 更新 README 的 Issue Domain 文档

**Files:**
- Modify: `README.md`
- Test: `tests/command_plans.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn issues_help_includes_issue_domain_write_commands() {
    let help = render_help(find_subcommand(&mut Cli::command(), "issues").clone());

    assert!(help.contains("create"));
    assert!(help.contains("update"));
    assert!(help.contains("time"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --locked issues_help_includes_issue_domain_write_commands`
Expected: FAIL，`issues` help 尚未暴露 phase 1 写命令。

- [ ] **Step 3: Write minimal implementation**

```md
### Issues

- `gitea-cli --json issues create ...`
- `gitea-cli --json issues update ...`
- `gitea-cli --json issues time list ...`
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --locked issues_help_includes_issue_domain_write_commands`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add README.md tests/command_plans.rs src/cli.rs
git commit -m "docs: document issue domain write commands"
```
