# Pull Request Write Commands Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 `gitea-cli` 补齐 Pull Request 主体写操作高层命令，覆盖创建、更新和合并。

**Architecture:** 延续现有 `src/cli.rs` 中 clap 参数定义加 `plan_pulls(...)` 映射到底层 MCP tool 的模式，不改动 `src/mcp.rs` 与输出层。实现按 TDD 分三批推进：先补 `pulls create`，再补 `pulls update` 和 `pulls merge`，最后收 README 与 help 文案。

**Tech Stack:** Rust 2024、clap 4、serde_json、cargo test

---

### Task 1: 为 `pulls create` 补失败测试与最小实现

**Files:**
- Modify: `tests/command_plans.rs`
- Modify: `src/cli.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn pulls_create_maps_to_pull_request_write_create() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "pulls",
        "create",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--head",
        "feature/pr-write",
        "--base",
        "main",
        "--title",
        "Add write commands",
        "--body",
        "details",
        "--label-id",
        "3",
        "--label-id",
        "5",
        "--draft",
        "--deadline",
        "2026-04-30T12:00:00Z",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "pull_request_write",
            serde_json::json!({
                "method": "create",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "head": "feature/pr-write",
                "base": "main",
                "title": "Add write commands",
                "body": "details",
                "labels": [3, 5],
                "draft": true,
                "deadline": "2026-04-30T12:00:00Z"
            })
        )
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test command_plans pulls_create_maps_to_pull_request_write_create -- --exact`
Expected: FAIL，提示 `pulls create` 尚未定义或映射不存在。

- [ ] **Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, Subcommand)]
pub enum PullsSubcommand {
    List(PullsListArgs),
    Get(PullTargetArgs),
    Diff(PullDiffArgs),
    Create(PullCreateArgs),
}

#[derive(Debug, Clone, Args)]
pub struct PullCreateArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    #[arg(long)]
    pub head: String,
    #[arg(long)]
    pub base: String,
    #[arg(long)]
    pub title: String,
    #[arg(long)]
    pub body: Option<String>,
    #[arg(long = "label-id")]
    pub label_ids: Vec<u64>,
    #[arg(long)]
    pub draft: bool,
    #[arg(long)]
    pub deadline: Option<String>,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test command_plans pulls_create_maps_to_pull_request_write_create -- --exact`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/command_plans.rs src/cli.rs
git commit -m "补齐 Pull Request 创建命令映射 (#2)"
```

### Task 2: 为 `pulls update` 补失败测试与最小实现

**Files:**
- Modify: `tests/command_plans.rs`
- Modify: `src/cli.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn pulls_update_maps_to_pull_request_write_update() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "pulls",
        "update",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--index",
        "12",
        "--title",
        "Updated title",
        "--state",
        "closed",
        "--base",
        "release/0.0.7",
        "--assignee",
        "fanbuz",
        "--label-id",
        "4",
        "--milestone",
        "7",
        "--deadline",
        "2026-04-30T12:00:00Z",
        "--remove-deadline",
        "--allow-maintainer-edit",
        "false",
        "--draft",
        "true",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "pull_request_write",
            serde_json::json!({
                "method": "update",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "index": 12,
                "title": "Updated title",
                "state": "closed",
                "base": "release/0.0.7",
                "assignee": "fanbuz",
                "labels": [4],
                "milestone": 7,
                "deadline": "2026-04-30T12:00:00Z",
                "remove_deadline": true,
                "allow_maintainer_edit": false,
                "draft": true
            })
        )
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test command_plans pulls_update_maps_to_pull_request_write_update -- --exact`
Expected: FAIL，提示 `pulls update` 尚未定义或映射不存在。

- [ ] **Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, Args)]
pub struct PullUpdateArgs {
    #[command(flatten)]
    pub target: PullTargetArgs,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub body: Option<String>,
    #[arg(long)]
    pub state: Option<String>,
    #[arg(long)]
    pub base: Option<String>,
    #[arg(long = "assignee")]
    pub assignees: Vec<String>,
    #[arg(long = "label-id")]
    pub label_ids: Vec<u64>,
    #[arg(long)]
    pub milestone: Option<u64>,
    #[arg(long)]
    pub deadline: Option<String>,
    #[arg(long)]
    pub remove_deadline: bool,
    #[arg(long = "allow-maintainer-edit")]
    pub allow_maintainer_edit: Option<bool>,
    #[arg(long)]
    pub draft: Option<bool>,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test command_plans pulls_update_maps_to_pull_request_write_update -- --exact`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/command_plans.rs src/cli.rs
git commit -m "补齐 Pull Request 更新命令映射 (#2)"
```

### Task 3: 为 `pulls merge` 补失败测试与最小实现

**Files:**
- Modify: `tests/command_plans.rs`
- Modify: `src/cli.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn pulls_merge_maps_to_pull_request_write_merge() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "pulls",
        "merge",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--index",
        "12",
        "--merge-style",
        "squash",
        "--title",
        "Merge PR",
        "--message",
        "merge details",
        "--delete-branch",
        "--force-merge",
        "--merge-when-checks-succeed",
        "--head-commit-id",
        "abcdef123456",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "pull_request_write",
            serde_json::json!({
                "method": "merge",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "index": 12,
                "merge_style": "squash",
                "title": "Merge PR",
                "message": "merge details",
                "delete_branch": true,
                "force_merge": true,
                "merge_when_checks_succeed": true,
                "head_commit_id": "abcdef123456"
            })
        )
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test command_plans pulls_merge_maps_to_pull_request_write_merge -- --exact`
Expected: FAIL，提示 `pulls merge` 尚未定义或映射不存在。

- [ ] **Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum PullMergeStyle {
    Merge,
    Rebase,
    RebaseMerge,
    Squash,
    FastForwardOnly,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test command_plans pulls_merge_maps_to_pull_request_write_merge -- --exact`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/command_plans.rs src/cli.rs
git commit -m "补齐 Pull Request 合并命令映射 (#2)"
```

### Task 4: 补充 help 文案与 README

**Files:**
- Modify: `tests/command_plans.rs`
- Modify: `README.md`
- Modify: `src/cli.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn pulls_help_includes_write_subcommands() {
    let mut root = Cli::command();
    let pulls_help = render_help(find_subcommand(&mut root, "pulls").clone());

    assert!(pulls_help.contains("create          创建 pull request"));
    assert!(pulls_help.contains("update          更新 pull request"));
    assert!(pulls_help.contains("merge           合并 pull request"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test command_plans pulls_help_includes_write_subcommands -- --exact`
Expected: FAIL，说明 help 还未暴露对应子命令说明。

- [ ] **Step 3: Write minimal implementation**

```markdown
### Pull Requests

- `gitea-cli --json pulls create --owner YOUR_ORG --repo YOUR_REPO --head feature-branch --base main --title "Add feature"`
  创建一个 Pull Request，适合从 feature 分支发起协作。

- `gitea-cli --json pulls update --owner YOUR_ORG --repo YOUR_REPO --index 12 --title "New title"`
  更新已有 Pull Request 的主体信息，可修改状态、分支、assignee、labels、milestone 和草稿状态。

- `gitea-cli --json pulls merge --owner YOUR_ORG --repo YOUR_REPO --index 12 --merge-style squash`
  合并一个 Pull Request，可控制 merge style、删除分支和检查通过后的自动合并行为。
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test command_plans pulls_help_includes_write_subcommands -- --exact`
Expected: PASS

- [ ] **Step 5: Run full verification**

Run: `cargo test`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add tests/command_plans.rs src/cli.rs README.md
git commit -m "完善 Pull Request 主体写操作文档与帮助信息 (#2)"
```
