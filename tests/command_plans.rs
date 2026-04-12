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
