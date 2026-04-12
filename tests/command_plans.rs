use gitea_cli::cli::{Cli, PlannedCommand, plan_command};

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
