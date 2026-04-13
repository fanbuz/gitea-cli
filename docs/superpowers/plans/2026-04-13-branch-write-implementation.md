# Branch Write Commands Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 `gitea-cli` 补齐分支管理高层命令，覆盖创建与删除分支。

**Architecture:** 延续现有 `src/cli.rs` 中 clap 参数定义加 `plan_repos(...)` 映射到底层 MCP tool 的模式，不改动 `src/mcp.rs` 与输出层。实现按 TDD 分三批推进：先补 `branch-create`，再补 `branch-delete` 的危险操作护栏，最后收 README 与 help 文案。

**Tech Stack:** Rust 2024、clap 4、serde_json、cargo test

---

### Task 1: 为 `repos branch-create` 补失败测试与最小实现

**Files:**
- Modify: `tests/command_plans.rs`
- Modify: `src/cli.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn repos_branch_create_maps_to_create_branch() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "repos",
        "branch-create",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--branch",
        "feature/new-command",
        "--from",
        "main",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "create_branch",
            serde_json::json!({
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "branch": "feature/new-command",
                "old_branch": "main"
            })
        )
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test command_plans repos_branch_create_maps_to_create_branch -- --exact`  
Expected: FAIL，提示 `repos branch-create` 尚未定义或映射不存在。

- [ ] **Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, Subcommand)]
pub enum ReposSubcommand {
    List(RepoListArgs),
    Branches(RepoTargetWithPageArgs),
    BranchCreate(RepoBranchCreateArgs),
    Tree(RepoTreeArgs),
}

#[derive(Debug, Clone, Args)]
pub struct RepoBranchCreateArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    #[arg(long)]
    pub branch: String,
    #[arg(long = "from")]
    pub old_branch: String,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test command_plans repos_branch_create_maps_to_create_branch -- --exact`  
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/command_plans.rs src/cli.rs
git commit -m "补齐分支创建命令映射 (#5)"
```

### Task 2: 为 `repos branch-delete` 补失败测试、护栏与最小实现

**Files:**
- Modify: `tests/command_plans.rs`
- Modify: `src/cli.rs`

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn repos_branch_delete_requires_yes() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "repos",
        "branch-delete",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--branch",
        "feature/new-command",
    ])
    .unwrap();

    let error = plan_command(&cli).unwrap_err();

    assert!(error.to_string().contains("--yes"));
}

#[test]
fn repos_branch_delete_maps_when_confirmed() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "repos",
        "branch-delete",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--branch",
        "feature/new-command",
        "--yes",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "delete_branch",
            serde_json::json!({
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "branch": "feature/new-command"
            })
        )
    );
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test command_plans repos_branch_delete_requires_yes repos_branch_delete_maps_when_confirmed`  
Expected: FAIL，提示 `repos branch-delete` 尚未定义或映射不存在。

- [ ] **Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, Subcommand)]
pub enum ReposSubcommand {
    List(RepoListArgs),
    Branches(RepoTargetWithPageArgs),
    BranchCreate(RepoBranchCreateArgs),
    BranchDelete(RepoBranchDeleteArgs),
    Tree(RepoTreeArgs),
}

#[derive(Debug, Clone, Args)]
pub struct RepoBranchDeleteArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    #[arg(long)]
    pub branch: String,
    #[arg(long)]
    pub yes: bool,
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test command_plans repos_branch_delete_requires_yes repos_branch_delete_maps_when_confirmed`  
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/command_plans.rs src/cli.rs
git commit -m "补齐分支删除命令映射 (#5)"
```

### Task 3: 为 Repos help 和 README 收失败测试与文档更新

**Files:**
- Modify: `tests/command_plans.rs`
- Modify: `README.md`
- Modify: `src/cli.rs`

- [ ] **Step 1: Write the failing help test**

```rust
#[test]
fn repos_help_includes_branch_write_subcommand_descriptions() {
    let mut root = Cli::command();
    let repos_help = render_help(find_subcommand(&mut root, "repos").clone());

    assert!(repos_help.contains("branch-create  创建仓库分支"));
    assert!(repos_help.contains("branch-delete  删除仓库分支"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test command_plans repos_help_includes_branch_write_subcommand_descriptions -- --exact`  
Expected: FAIL，help 中还没有两条分支写操作子命令。

- [ ] **Step 3: Update help 文案与 README**

```md
- `gitea-cli --json repos branch-create --owner YOUR_ORG --repo YOUR_REPO --branch feature/new-command --from main`
  基于指定已有分支创建新分支。

- `gitea-cli --json repos branch-delete --owner YOUR_ORG --repo YOUR_REPO --branch feature/new-command --yes`
  删除指定仓库分支，属于危险操作，必须显式传 `--yes`。
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test command_plans repos_help_includes_branch_write_subcommand_descriptions -- --exact`  
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/command_plans.rs README.md src/cli.rs
git commit -m "补齐分支命令文档与帮助信息 (#5)"
```

### Task 4: 完整验证并提交收口

**Files:**
- Verify only: `src/cli.rs`
- Verify only: `tests/command_plans.rs`
- Verify only: `README.md`

- [ ] **Step 1: Run targeted Repos command plan tests**

Run: `cargo test --test command_plans repos_`  
Expected: PASS，包含现有只读与新增分支写操作测试。

- [ ] **Step 2: Run full test suite**

Run: `cargo test`  
Expected: PASS，无失败测试。

- [ ] **Step 3: Review diff before commit handoff**

Run: `git diff -- src/cli.rs tests/command_plans.rs README.md docs/superpowers/specs/2026-04-13-branch-write-design.md docs/superpowers/plans/2026-04-13-branch-write-implementation.md`  
Expected: 仅包含 `#5` 相关命令、文档、测试与设计计划文件。

- [ ] **Step 4: Commit docs and implementation**

```bash
git add docs/superpowers/specs/2026-04-13-branch-write-design.md docs/superpowers/plans/2026-04-13-branch-write-implementation.md src/cli.rs tests/command_plans.rs README.md
git commit -m "补齐分支管理高层命令 (#5)"
```
