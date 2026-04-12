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
fn top_level_help_includes_command_descriptions() {
    let help = render_help(Cli::command());

    assert!(help.contains("doctor    检查 gitea-cli 与底层 Gitea MCP 配置是否可用"));
    assert!(help.contains("issues    查询 issue 列表、详情、评论与跨仓库搜索"));
    assert!(help.contains("releases  查询仓库 release 列表、最新版本和单个 release"));
}

#[test]
fn issues_help_includes_subcommand_descriptions() {
    let mut root = Cli::command();
    let issues_help = render_help(find_subcommand(&mut root, "issues").clone());

    assert!(issues_help.contains("list      列出仓库 issue 列表"));
    assert!(issues_help.contains("get       读取单个 issue 详情"));
    assert!(issues_help.contains("comments  读取单个 issue 的评论列表"));
    assert!(issues_help.contains("search    按关键词跨仓库搜索 issue 或 pull request"));
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
