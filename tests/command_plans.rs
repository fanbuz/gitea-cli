use clap::CommandFactory;
use gitea_cli::cli::{Cli, PlannedCommand, plan_command};

fn render_help(mut command: clap::Command) -> String {
    let mut output = Vec::new();
    command.write_long_help(&mut output).unwrap();
    String::from_utf8(output).unwrap()
}

fn find_subcommand<'a>(command: &'a mut clap::Command, name: &str) -> &'a mut clap::Command {
    command.find_subcommand_mut(name).unwrap()
}

fn help_has_command_description(help: &str, command: &str, description: &str) -> bool {
    help.lines()
        .any(|line| line.contains(command) && line.contains(description))
}

#[test]
fn issue_get_maps_to_issue_read() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "--json",
        "issues",
        "get",
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
            "issue_read",
            serde_json::json!({
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "index": 524,
                "method": "get"
            })
        )
    );
}

#[test]
fn actions_log_preview_maps_to_actions_run_read() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "actions",
        "log-preview",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--job-id",
        "456",
        "--tail-lines",
        "20",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "actions_run_read",
            serde_json::json!({
                "method": "get_job_log_preview",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "job_id": 456,
                "tail_lines": 20
            })
        )
    );
}

#[test]
fn repos_list_with_owner_uses_org_repos() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "repos",
        "list",
        "--owner",
        "XINTUKJ",
        "--page",
        "2",
        "--page-size",
        "50",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "list_org_repos",
            serde_json::json!({
                "org": "XINTUKJ",
                "page": 2,
                "pageSize": 50
            })
        )
    );
}

#[test]
fn resolve_issue_url_extracts_coordinates() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "resolve",
        "issue",
        "https://code.example.com/XINTUKJ/simba-ehr-frontend/issues/524",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::resolve(serde_json::json!({
            "owner": "XINTUKJ",
            "repo": "simba-ehr-frontend",
            "index": 524
        }))
    );
}

#[test]
fn releases_latest_maps_to_get_latest_release() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "releases",
        "latest",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "get_latest_release",
            serde_json::json!({
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend"
            })
        )
    );
}

#[test]
fn releases_list_maps_to_list_releases() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "releases",
        "list",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--page",
        "3",
        "--page-size",
        "25",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "list_releases",
            serde_json::json!({
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "page": 3,
                "perPage": 25
            })
        )
    );
}

#[test]
fn releases_get_maps_to_get_release() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "releases",
        "get",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--id",
        "12",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "get_release",
            serde_json::json!({
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "id": 12
            })
        )
    );
}

#[test]
fn tags_get_maps_to_get_tag() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "tags",
        "get",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--tag",
        "v0.0.2",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "get_tag",
            serde_json::json!({
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "tag_name": "v0.0.2"
            })
        )
    );
}

#[test]
fn tags_list_maps_to_list_tags() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "tags",
        "list",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--page",
        "2",
        "--page-size",
        "30",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "list_tags",
            serde_json::json!({
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "page": 2,
                "perPage": 30
            })
        )
    );
}

#[test]
fn commits_list_maps_to_list_commits() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "commits",
        "list",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--sha",
        "main",
        "--path",
        "src/app.rs",
        "--page",
        "2",
        "--page-size",
        "40",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "list_commits",
            serde_json::json!({
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "sha": "main",
                "path": "src/app.rs",
                "page": 2,
                "perPage": 40
            })
        )
    );
}

#[test]
fn commits_get_maps_to_get_commit() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "commits",
        "get",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--sha",
        "abcdef123456",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "get_commit",
            serde_json::json!({
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "sha": "abcdef123456"
            })
        )
    );
}

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

#[test]
fn top_level_help_includes_command_descriptions() {
    let help = render_help(Cli::command());

    assert!(help_has_command_description(
        &help,
        "doctor",
        "检查 gitea-cli 与底层 Gitea MCP 配置是否可用"
    ));
    assert!(help_has_command_description(
        &help,
        "issues",
        "管理 issue、评论、labels 与 time tracking"
    ));
    assert!(help_has_command_description(
        &help,
        "releases",
        "查询仓库 release 列表、最新版本和单个 release"
    ));
}

#[test]
fn top_level_help_includes_current_version() {
    let help = render_help(Cli::command());

    assert!(help.contains(&format!("当前版本: {}", env!("CARGO_PKG_VERSION"))));
}

#[test]
fn issues_help_includes_subcommand_descriptions() {
    let mut root = Cli::command();
    let issues_help = render_help(find_subcommand(&mut root, "issues").clone());

    assert!(issues_help.contains("list            列出仓库 issue 列表"));
    assert!(issues_help.contains("get             读取单个 issue 详情"));
    assert!(issues_help.contains("comments        读取单个 issue 的评论列表"));
    assert!(issues_help.contains("search          按关键词跨仓库搜索 issue 或 pull request"));
}

#[test]
fn issues_list_help_includes_option_descriptions() {
    let mut root = Cli::command();
    let issues = find_subcommand(&mut root, "issues");
    let issues_list_help = render_help(find_subcommand(issues, "list").clone());

    assert!(issues_list_help.contains("--owner <OWNER>"));
    assert!(issues_list_help.contains("Gitea 仓库所属 owner 或组织"));
    assert!(issues_list_help.contains("--repo <REPO>"));
    assert!(issues_list_help.contains("Gitea 仓库名"));
    assert!(issues_list_help.contains("--state <STATE>"));
    assert!(issues_list_help.contains("Issue 状态过滤，默认 open"));
    assert!(issues_list_help.contains("--page-size <PAGE_SIZE>"));
    assert!(issues_list_help.contains("每页返回条数"));
}

#[test]
fn pulls_help_includes_write_subcommands() {
    let mut root = Cli::command();
    let pulls_help = render_help(find_subcommand(&mut root, "pulls").clone());

    assert!(help_has_command_description(
        &pulls_help,
        "create",
        "创建 pull request"
    ));
    assert!(help_has_command_description(
        &pulls_help,
        "update",
        "更新 pull request"
    ));
    assert!(help_has_command_description(
        &pulls_help,
        "merge",
        "合并 pull request"
    ));
}

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
fn issues_comment_edit_maps_to_issue_write_edit_comment() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "issues",
        "comment-edit",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--index",
        "524",
        "--comment-id",
        "88",
        "--body",
        "edited body",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "issue_write",
            serde_json::json!({
                "method": "edit_comment",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "index": 524,
                "commentID": 88,
                "body": "edited body"
            })
        )
    );
}

#[test]
fn issues_labels_maps_to_issue_read_get_labels() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "issues",
        "labels",
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
            "issue_read",
            serde_json::json!({
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "index": 524,
                "method": "get_labels"
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

#[test]
fn issues_label_remove_requires_yes() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "issues",
        "label-remove",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--index",
        "524",
        "--label-id",
        "2",
    ])
    .unwrap();

    let error = plan_command(&cli).unwrap_err();

    assert!(error.to_string().contains("--yes"));
}

#[test]
fn issues_label_remove_maps_when_confirmed() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "issues",
        "label-remove",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--index",
        "524",
        "--label-id",
        "2",
        "--yes",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "issue_write",
            serde_json::json!({
                "method": "remove_label",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "index": 524,
                "label_id": 2
            })
        )
    );
}

#[test]
fn issues_labels_replace_maps_to_issue_write_replace_labels() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "issues",
        "labels-replace",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--index",
        "524",
        "--label-id",
        "4",
        "--label-id",
        "5",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "issue_write",
            serde_json::json!({
                "method": "replace_labels",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "index": 524,
                "labels": [4, 5]
            })
        )
    );
}

#[test]
fn issues_labels_clear_requires_yes() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "issues",
        "labels-clear",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--index",
        "524",
    ])
    .unwrap();

    let error = plan_command(&cli).unwrap_err();

    assert!(error.to_string().contains("--yes"));
}

#[test]
fn issues_labels_clear_maps_when_confirmed() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "issues",
        "labels-clear",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--index",
        "524",
        "--yes",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "issue_write",
            serde_json::json!({
                "method": "clear_labels",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "index": 524
            })
        )
    );
}

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
fn issues_time_start_maps_to_timetracking_write_start_stopwatch() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "issues",
        "time",
        "start",
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
            "timetracking_write",
            serde_json::json!({
                "method": "start_stopwatch",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "index": 524
            })
        )
    );
}

#[test]
fn issues_time_stop_maps_to_timetracking_write_stop_stopwatch() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "issues",
        "time",
        "stop",
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
            "timetracking_write",
            serde_json::json!({
                "method": "stop_stopwatch",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "index": 524
            })
        )
    );
}

#[test]
fn issues_time_reset_stopwatch_requires_yes() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "issues",
        "time",
        "reset-stopwatch",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--index",
        "524",
    ])
    .unwrap();

    let error = plan_command(&cli).unwrap_err();

    assert!(error.to_string().contains("--yes"));
}

#[test]
fn issues_time_reset_stopwatch_maps_when_confirmed() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "issues",
        "time",
        "reset-stopwatch",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--index",
        "524",
        "--yes",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "timetracking_write",
            serde_json::json!({
                "method": "delete_stopwatch",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "index": 524
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

#[test]
fn issues_time_delete_requires_yes() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "issues",
        "time",
        "delete",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--index",
        "524",
        "--id",
        "77",
    ])
    .unwrap();

    let error = plan_command(&cli).unwrap_err();

    assert!(error.to_string().contains("--yes"));
}

#[test]
fn issues_time_delete_maps_when_confirmed() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "issues",
        "time",
        "delete",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--index",
        "524",
        "--id",
        "77",
        "--yes",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "timetracking_write",
            serde_json::json!({
                "method": "delete_time",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "index": 524,
                "id": 77
            })
        )
    );
}

#[test]
fn issues_help_includes_phase1_subcommand_descriptions() {
    let mut root = Cli::command();
    let issues_help = render_help(find_subcommand(&mut root, "issues").clone());

    assert!(issues_help.contains("create          创建 issue"));
    assert!(issues_help.contains("update          更新 issue"));
    assert!(issues_help.contains("comment-add     为 issue 添加评论"));
    assert!(issues_help.contains("comment-edit    编辑 issue 评论"));
    assert!(issues_help.contains("labels          读取 issue 当前 labels"));
    assert!(issues_help.contains("time            读取或写入 issue time tracking"));
}

#[test]
fn issues_time_help_includes_subcommand_descriptions() {
    let mut root = Cli::command();
    let issues = find_subcommand(&mut root, "issues");
    let time_help = render_help(find_subcommand(issues, "time").clone());

    assert!(time_help.contains("list             读取 issue time tracking 记录"));
    assert!(time_help.contains("start            启动 issue stopwatch"));
    assert!(time_help.contains("reset-stopwatch  清空 issue stopwatch"));
    assert!(time_help.contains("delete           删除一条 issue time 记录"));
}

#[test]
fn labels_repo_list_maps_to_label_read_list_repo_labels() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "labels",
        "repo-list",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--page",
        "2",
        "--page-size",
        "50",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "label_read",
            serde_json::json!({
                "method": "list_repo_labels",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "page": 2,
                "perPage": 50
            })
        )
    );
}

#[test]
fn labels_repo_get_maps_to_label_read_get_repo_label() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "labels",
        "repo-get",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--id",
        "9",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "label_read",
            serde_json::json!({
                "method": "get_repo_label",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "id": 9
            })
        )
    );
}

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
        "--description",
        "important",
        "--archived",
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
                "color": "#ff0000",
                "description": "important",
                "is_archived": true
            })
        )
    );
}

#[test]
fn labels_repo_edit_maps_to_label_write_edit_repo_label() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "labels",
        "repo-edit",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--id",
        "9",
        "--name",
        "bugfix",
        "--color",
        "#00ff00",
        "--description",
        "updated",
        "--archived",
        "false",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "label_write",
            serde_json::json!({
                "method": "edit_repo_label",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "id": 9,
                "name": "bugfix",
                "color": "#00ff00",
                "description": "updated",
                "is_archived": false
            })
        )
    );
}

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

#[test]
fn labels_repo_delete_maps_when_confirmed() {
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
        "--yes",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "label_write",
            serde_json::json!({
                "method": "delete_repo_label",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "id": 9
            })
        )
    );
}

#[test]
fn labels_org_list_maps_to_label_read_list_org_labels() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "labels",
        "org-list",
        "--org",
        "XINTUKJ",
        "--page",
        "3",
        "--page-size",
        "20",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "label_read",
            serde_json::json!({
                "method": "list_org_labels",
                "org": "XINTUKJ",
                "page": 3,
                "perPage": 20
            })
        )
    );
}

#[test]
fn labels_org_create_maps_to_label_write_create_org_label() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "labels",
        "org-create",
        "--org",
        "XINTUKJ",
        "--name",
        "needs-review",
        "--color",
        "#123456",
        "--description",
        "org label",
        "--exclusive",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "label_write",
            serde_json::json!({
                "method": "create_org_label",
                "org": "XINTUKJ",
                "name": "needs-review",
                "color": "#123456",
                "description": "org label",
                "exclusive": true
            })
        )
    );
}

#[test]
fn labels_org_edit_maps_to_label_write_edit_org_label() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "labels",
        "org-edit",
        "--org",
        "XINTUKJ",
        "--id",
        "12",
        "--name",
        "ready",
        "--color",
        "#654321",
        "--description",
        "updated org label",
        "--exclusive",
        "false",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "label_write",
            serde_json::json!({
                "method": "edit_org_label",
                "org": "XINTUKJ",
                "id": 12,
                "name": "ready",
                "color": "#654321",
                "description": "updated org label",
                "exclusive": false
            })
        )
    );
}

#[test]
fn labels_org_delete_requires_yes() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "labels",
        "org-delete",
        "--org",
        "XINTUKJ",
        "--id",
        "12",
    ])
    .unwrap();

    let error = plan_command(&cli).unwrap_err();

    assert!(error.to_string().contains("--yes"));
}

#[test]
fn labels_org_delete_maps_when_confirmed() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "labels",
        "org-delete",
        "--org",
        "XINTUKJ",
        "--id",
        "12",
        "--yes",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "label_write",
            serde_json::json!({
                "method": "delete_org_label",
                "org": "XINTUKJ",
                "id": 12
            })
        )
    );
}

#[test]
fn milestones_list_maps_to_milestone_read_list() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "milestones",
        "list",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--state",
        "open",
        "--name",
        "v0",
        "--page",
        "2",
        "--page-size",
        "40",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "milestone_read",
            serde_json::json!({
                "method": "list",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "state": "open",
                "name": "v0",
                "page": 2,
                "perPage": 40
            })
        )
    );
}

#[test]
fn milestones_get_maps_to_milestone_read_get() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "milestones",
        "get",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--id",
        "6",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "milestone_read",
            serde_json::json!({
                "method": "get",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "id": 6
            })
        )
    );
}

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
        "--description",
        "phase 2",
        "--due-on",
        "2026-04-30T00:00:00Z",
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
                "title": "v0.0.6",
                "description": "phase 2",
                "due_on": "2026-04-30T00:00:00Z"
            })
        )
    );
}

#[test]
fn milestones_edit_maps_to_milestone_write_edit() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "milestones",
        "edit",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--id",
        "6",
        "--title",
        "v0.0.6-1",
        "--description",
        "updated phase 2",
        "--due-on",
        "2026-05-01T00:00:00Z",
        "--state",
        "closed",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "milestone_write",
            serde_json::json!({
                "method": "edit",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "id": 6,
                "title": "v0.0.6-1",
                "description": "updated phase 2",
                "due_on": "2026-05-01T00:00:00Z",
                "state": "closed"
            })
        )
    );
}

#[test]
fn milestones_delete_requires_yes() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "milestones",
        "delete",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--id",
        "6",
    ])
    .unwrap();

    let error = plan_command(&cli).unwrap_err();

    assert!(error.to_string().contains("--yes"));
}

#[test]
fn milestones_delete_maps_when_confirmed() {
    let cli = Cli::try_parse_from([
        "gitea-cli",
        "milestones",
        "delete",
        "--owner",
        "XINTUKJ",
        "--repo",
        "simba-ehr-frontend",
        "--id",
        "6",
        "--yes",
    ])
    .unwrap();

    let planned = plan_command(&cli).unwrap();

    assert_eq!(
        planned,
        PlannedCommand::tool_call(
            "milestone_write",
            serde_json::json!({
                "method": "delete",
                "owner": "XINTUKJ",
                "repo": "simba-ehr-frontend",
                "id": 6
            })
        )
    );
}

#[test]
fn top_level_help_includes_label_and_milestone_commands() {
    let help = render_help(Cli::command());

    assert!(help.contains("labels"));
    assert!(help.contains("milestones"));
}

#[test]
fn labels_help_includes_subcommand_descriptions() {
    let mut root = Cli::command();
    let labels_help = render_help(find_subcommand(&mut root, "labels").clone());

    assert!(help_has_command_description(
        &labels_help,
        "repo-list",
        "列出仓库 labels"
    ));
    assert!(help_has_command_description(
        &labels_help,
        "repo-create",
        "创建仓库 label"
    ));
    assert!(help_has_command_description(
        &labels_help,
        "org-list",
        "列出组织 labels"
    ));
    assert!(help_has_command_description(
        &labels_help,
        "org-delete",
        "删除组织 label"
    ));
}

#[test]
fn milestones_help_includes_subcommand_descriptions() {
    let mut root = Cli::command();
    let milestones_help = render_help(find_subcommand(&mut root, "milestones").clone());

    assert!(milestones_help.contains("list    列出仓库 milestones"));
    assert!(milestones_help.contains("get     读取单个 milestone"));
    assert!(milestones_help.contains("create  创建 milestone"));
    assert!(milestones_help.contains("delete  删除 milestone"));
}
