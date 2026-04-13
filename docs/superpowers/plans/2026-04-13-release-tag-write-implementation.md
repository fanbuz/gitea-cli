# Release And Tag Write Commands Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 为 `gitea-cli` 补齐 release 与 tag 的创建、删除高层命令，并同步补完 help 与 README。

**Architecture:** 延续现有 `src/cli.rs` 中 clap 参数定义加 `plan_releases(...)` / `plan_tags(...)` 映射到底层 MCP tool 的模式，不改动会话层与输出归一化逻辑。实现按 TDD 分四段推进：release create、release delete、tag write、help 与 README 收口。

**Tech Stack:** Rust 2024、clap 4、serde_json、cargo test

---

### Task 1: 为 `releases create` 补失败测试与最小实现

**Files:**
- Modify: `tests/command_plans.rs`
- Modify: `src/cli.rs`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn releases_create_maps_to_create_release() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "releases",
        "create",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--tag",
        "v0.0.7",
        "--title",
        "v0.0.7",
        "--target",
        "main",
        "--body",
        "release notes",
        "--draft",
        "--pre-release",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "create_release",
            serde_json::json!({
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "tag_name": "v0.0.7",
                "title": "v0.0.7",
                "target": "main",
                "body": "release notes",
                "is_draft": true,
                "is_pre_release": true
            })
        )
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test command_plans releases_create_maps_to_create_release -- --exact`  
Expected: FAIL，提示 `releases create` 尚未定义或映射不存在。

- [ ] **Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, Subcommand)]
pub enum ReleasesSubcommand {
    List(RepoTargetWithPageArgs),
    Latest(RepoTargetArgs),
    Get(ReleaseTargetArgs),
    Create(ReleaseCreateArgs),
}

#[derive(Debug, Clone, Args)]
pub struct ReleaseCreateArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    #[arg(long = "tag")]
    pub tag_name: String,
    #[arg(long)]
    pub title: String,
    #[arg(long)]
    pub target_ref: String,
    #[arg(long)]
    pub body: Option<String>,
    #[arg(long)]
    pub draft: bool,
    #[arg(long = "pre-release")]
    pub pre_release: bool,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test command_plans releases_create_maps_to_create_release -- --exact`  
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/command_plans.rs src/cli.rs
git commit -m "补齐 release 创建命令映射 (#7)"
```

### Task 2: 为 `releases delete` 补失败测试、护栏与最小实现

**Files:**
- Modify: `tests/command_plans.rs`
- Modify: `src/cli.rs`

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn releases_delete_requires_yes() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "releases",
        "delete",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--id",
        "12",
    ])
    .unwrap();

    let error = plan_command(&cli).unwrap_err();

    assert!(error.to_string().contains("--yes"));
}

#[test]
fn releases_delete_maps_when_confirmed() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "releases",
        "delete",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--id",
        "12",
        "--yes",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "delete_release",
            serde_json::json!({
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "id": 12
            })
        )
    );
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test command_plans releases_delete_`  
Expected: FAIL，提示 `releases delete` 尚未定义或映射不存在。

- [ ] **Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, Subcommand)]
pub enum ReleasesSubcommand {
    List(RepoTargetWithPageArgs),
    Latest(RepoTargetArgs),
    Get(ReleaseTargetArgs),
    Create(ReleaseCreateArgs),
    Delete(ReleaseDeleteArgs),
}

#[derive(Debug, Clone, Args)]
pub struct ReleaseDeleteArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    #[arg(long)]
    pub id: u64,
    #[arg(long)]
    pub yes: bool,
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test command_plans releases_delete_`  
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/command_plans.rs src/cli.rs
git commit -m "补齐 release 删除命令映射 (#7)"
```

### Task 3: 为 `tags create/delete` 补失败测试与最小实现

**Files:**
- Modify: `tests/command_plans.rs`
- Modify: `src/cli.rs`

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn tags_create_maps_to_create_tag() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "tags",
        "create",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--tag",
        "v0.0.7",
        "--target",
        "main",
        "--message",
        "annotated tag",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "create_tag",
            serde_json::json!({
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "tag_name": "v0.0.7",
                "target": "main",
                "message": "annotated tag"
            })
        )
    );
}

#[test]
fn tags_delete_requires_yes() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "tags",
        "delete",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--tag",
        "v0.0.7",
    ])
    .unwrap();

    let error = plan_command(&cli).unwrap_err();

    assert!(error.to_string().contains("--yes"));
}

#[test]
fn tags_delete_maps_when_confirmed() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "tags",
        "delete",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--tag",
        "v0.0.7",
        "--yes",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "delete_tag",
            serde_json::json!({
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "tag_name": "v0.0.7"
            })
        )
    );
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test command_plans tags_`  
Expected: FAIL，提示 `tags create` / `tags delete` 尚未定义或映射不存在。

- [ ] **Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, Subcommand)]
pub enum TagsSubcommand {
    List(RepoTargetWithPageArgs),
    Get(TagTargetArgs),
    Create(TagCreateArgs),
    Delete(TagDeleteArgs),
}

#[derive(Debug, Clone, Args)]
pub struct TagCreateArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    #[arg(long = "tag")]
    pub tag_name: String,
    #[arg(long)]
    pub target_ref: Option<String>,
    #[arg(long)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct TagDeleteArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    #[arg(long = "tag")]
    pub tag_name: String,
    #[arg(long)]
    pub yes: bool,
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test command_plans tags_`  
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/command_plans.rs src/cli.rs
git commit -m "补齐 tag 写命令映射 (#7)"
```

### Task 4: 为 help 与 README 收失败测试和文档更新

**Files:**
- Modify: `tests/command_plans.rs`
- Modify: `src/cli.rs`
- Modify: `README.md`

- [ ] **Step 1: Write the failing help tests**

```rust
#[test]
fn releases_help_includes_write_subcommand_descriptions() {
    let mut root = Cli::command();
    let releases_help = render_help(find_subcommand(&mut root, "releases").clone());

    assert!(releases_help.contains("create          创建 release"));
    assert!(releases_help.contains("delete          删除 release"));
}

#[test]
fn tags_help_includes_write_subcommand_descriptions() {
    let mut root = Cli::command();
    let tags_help = render_help(find_subcommand(&mut root, "tags").clone());

    assert!(tags_help.contains("create          创建 tag"));
    assert!(tags_help.contains("delete          删除 tag"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test command_plans write_subcommand_descriptions`  
Expected: FAIL，help 中还没有两条写操作子命令。

- [ ] **Step 3: Update help 文案与 README**

```md
- `gitea-cli --json releases create --owner YOUR_ORG --repo YOUR_REPO --tag v0.0.7 --title "v0.0.7" --target main`
  创建一个 release，可附带 body、draft 与 pre-release 标记。

- `gitea-cli --json releases delete --owner YOUR_ORG --repo YOUR_REPO --id 12 --yes`
  删除一个 release，属于危险操作，必须显式传 `--yes`。

- `gitea-cli --json tags create --owner YOUR_ORG --repo YOUR_REPO --tag v0.0.7 --target main`
  创建一个 tag，可选附带注释 message。

- `gitea-cli --json tags delete --owner YOUR_ORG --repo YOUR_REPO --tag v0.0.7 --yes`
  删除一个 tag，属于危险操作，必须显式传 `--yes`。
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --test command_plans write_subcommand_descriptions`  
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add tests/command_plans.rs src/cli.rs README.md
git commit -m "补齐 release 与 tag 命令文档 (#7)"
```

### Task 5: 完整验证并提交收口

**Files:**
- Verify only: `src/cli.rs`
- Verify only: `tests/command_plans.rs`
- Verify only: `README.md`

- [ ] **Step 1: Run targeted Releases tests**

Run: `cargo test --test command_plans releases_`  
Expected: PASS，包含只读与新增写操作测试。

- [ ] **Step 2: Run targeted Tags tests**

Run: `cargo test --test command_plans tags_`  
Expected: PASS，包含只读与新增写操作测试。

- [ ] **Step 3: Run full test suite**

Run: `cargo test`  
Expected: PASS，无失败测试。

- [ ] **Step 4: Review diff before handoff**

Run: `git diff -- src/cli.rs tests/command_plans.rs README.md docs/superpowers/specs/2026-04-13-release-tag-write-design.md docs/superpowers/plans/2026-04-13-release-tag-write-implementation.md`  
Expected: 仅包含 `#7` 相关变更，无意外文件。
