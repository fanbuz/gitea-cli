use clap::CommandFactory;
use gitea_cli::cli::{Cli, PlannedCommand, plan_command};
use serde_json::{Value, json};

fn plan(args: &[&str]) -> anyhow::Result<PlannedCommand> {
    let cli = Cli::try_parse_from(std::iter::once("gitea-cli").chain(args.iter().copied()))?;
    plan_command(&cli)
}

fn params(args: &[&str], tool: &str) -> Value {
    match plan(args).unwrap() {
        PlannedCommand::ToolCall {
            tool: actual,
            params,
            filter: None,
        } => {
            assert_eq!(actual, tool);
            params
        }
        other => panic!("unexpected: {other:?}"),
    }
}

#[test]
fn new_command_help_is_consistent() {
    Cli::command().debug_assert();
    for (group, commands) in [
        ("orgs", vec!["create"]),
        ("repos", vec!["files-write"]),
        (
            "issues",
            vec![
                "comment-get",
                "blocked-by",
                "blocking",
                "dependency-add",
                "dependency-remove",
            ],
        ),
        ("pulls", vec!["comments", "status"]),
        (
            "projects",
            vec![
                "list",
                "get",
                "columns",
                "column-issues",
                "create",
                "update",
                "delete",
                "column-create",
                "column-update",
                "column-delete",
                "issue-add",
                "issue-remove",
            ],
        ),
    ] {
        for command in commands {
            let error = Cli::try_parse_from(["gitea-cli", group, command, "--help"]).unwrap_err();
            assert_eq!(error.kind(), clap::error::ErrorKind::DisplayHelp);
        }
    }
}

#[test]
fn organization_creation_maps_all_fields_and_rejects_invalid_visibility() {
    assert_eq!(
        params(
            &[
                "orgs",
                "create",
                "--username",
                "team",
                "--full-name",
                "Team",
                "--description",
                "",
                "--website",
                "https://example.com",
                "--visibility",
                "private"
            ],
            "create_organization"
        ),
        json!({"username":"team","full_name":"Team","description":"","website":"https://example.com","visibility":"private"})
    );
    assert_eq!(
        params(
            &["orgs", "create", "--username", "team"],
            "create_organization"
        ),
        json!({"username":"team"})
    );
    assert!(
        plan(&[
            "orgs",
            "create",
            "--username",
            "team",
            "--visibility",
            "unknown"
        ])
        .is_err()
    );
}

#[test]
fn line_ranges_are_positive_ordered_and_optional() {
    let base = [
        "repos",
        "file",
        "--owner",
        "o",
        "--repo",
        "r",
        "--path",
        "README.md",
    ];
    let p = params(
        &[&base[..], &["--start-line", "2", "--end-line", "4"]].concat(),
        "get_file_contents",
    );
    assert_eq!(p["start_line"], 2);
    assert_eq!(p["end_line"], 4);
    for suffix in [
        vec!["--start-line", "0"],
        vec!["--start-line", "5", "--end-line", "4"],
    ] {
        assert!(plan(&[&base[..], &suffix].concat()).is_err());
    }
    assert!(
        params(&base, "get_file_contents")
            .get("start_line")
            .is_none()
    );
}

#[test]
fn edits_preserve_literal_strings_and_require_sha_for_single_file_update() {
    let edits = r#"[{"old_string":"a\nb","new_string":"中文\n$HOME","replace_all":true}]"#;
    let base = [
        "repos",
        "file-update",
        "--owner",
        "o",
        "--repo",
        "r",
        "--path",
        "a",
        "--branch",
        "main",
        "--message",
        "edit",
        "--edits",
        edits,
    ];
    assert!(plan(&base).is_err());
    let args = [&base[..], &["--sha", "abc"]].concat();
    let p = params(&args, "create_or_update_file");
    assert_eq!(p["edits"], serde_json::from_str::<Value>(edits).unwrap());
    assert!(p.get("content").is_none());
    assert!(plan(&[&args[..], &["--content", "x"]].concat()).is_err());
}

#[test]
fn batch_files_support_create_edit_rename_delete_and_file_input() {
    let files = json!([
        {"path":"new", "content":""},
        {"path":"edit", "edits":[{"old_string":"x","new_string":"y"}]},
        {"path":"renamed", "from_path":"old", "sha":"abc"},
        {"path":"gone", "delete":true, "sha":"def"}
    ]);
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(file.path(), files.to_string()).unwrap();
    let input = format!("@{}", file.path().display());
    let base = [
        "repos",
        "files-write",
        "--owner",
        "o",
        "--repo",
        "r",
        "--branch",
        "main",
        "--message",
        "batch",
        "--files",
        &input,
        "--new-branch",
        "feature",
    ];
    assert!(plan(&base).unwrap_err().to_string().contains("--yes"));
    let p = params(&[&base[..], &["--yes"]].concat(), "create_or_update_file");
    assert_eq!(p["files"], files);
    assert_eq!(p["new_branch_name"], "feature");
}

#[test]
fn invalid_batch_operations_fail_before_any_remote_call() {
    for files in [
        "[]",
        "{}",
        r#"[{"path":"a"}]"#,
        r#"[{"path":"a","delete":true}]"#,
        r#"[{"path":"a","from_path":"b"}]"#,
        r#"[{"path":"a","delete":true,"sha":"x","content":""}]"#,
        r#"[{"path":"a","content":"","edits":[{"old_string":"x","new_string":"y"}]}]"#,
        r#"[{"path":"a","edits":[]}]"#,
        r#"[{"path":"a","edits":[{"old_string":"","new_string":"x"}]}]"#,
        r#"[{"path":"a","content":""},{"path":"b","from_path":"a","sha":"x"}]"#,
        r#"[{"path":"a","content":"","typo":true}]"#,
        r#"[{"path":"a","delete":"true","sha":"x"}]"#,
    ] {
        assert!(
            plan(&[
                "repos",
                "files-write",
                "--owner",
                "o",
                "--repo",
                "r",
                "--branch",
                "main",
                "--message",
                "batch",
                "--files",
                files,
                "--yes"
            ])
            .is_err(),
            "{files}"
        );
    }
}

#[test]
fn collaboration_reads_and_filters_match_upstream_contract() {
    assert_eq!(
        params(
            &[
                "issues",
                "comment-get",
                "--owner",
                "o",
                "--repo",
                "r",
                "--comment-id",
                "9"
            ],
            "issue_read"
        ),
        json!({"method":"get_comment","owner":"o","repo":"r","comment_id":9})
    );
    for (command, method) in [
        ("blocked-by", "get_blocked_by"),
        ("blocking", "get_blocking"),
    ] {
        assert_eq!(
            params(
                &[
                    "issues",
                    command,
                    "--owner",
                    "o",
                    "--repo",
                    "r",
                    "--index",
                    "2",
                    "--page",
                    "3",
                    "--page-size",
                    "4"
                ],
                "issue_read"
            ),
            json!({"method":method,"owner":"o","repo":"r","issue_number":2,"page":3,"per_page":4})
        );
    }
    assert_eq!(
        params(
            &[
                "pulls", "comments", "--owner", "o", "--repo", "r", "--index", "2"
            ],
            "pull_request_read"
        ),
        json!({"method":"get_comments","owner":"o","repo":"r","pull_number":2,"page":1,"per_page":30})
    );
    let base = [
        "pulls", "status", "--owner", "o", "--repo", "r", "--index", "2",
    ];
    assert!(
        params(&base, "pull_request_read")
            .get("wait_seconds")
            .is_none()
    );
    assert_eq!(
        params(
            &[&base[..], &["--wait-seconds", "45"]].concat(),
            "pull_request_read"
        )["wait_seconds"],
        45
    );
    assert!(plan(&[&base[..], &["--wait-seconds", "46"]].concat()).is_err());
    assert_eq!(
        params(
            &[
                "actions",
                "runs",
                "--owner",
                "o",
                "--repo",
                "r",
                "--branch",
                "feature",
                "--head-sha",
                "abc",
                "--status",
                "success"
            ],
            "actions_run_read"
        ),
        json!({"method":"list_runs","owner":"o","repo":"r","page":1,"per_page":30,"branch":"feature","head_sha":"abc","status":"success"})
    );
}

#[test]
fn dependency_direction_and_cross_repo_target_are_preserved() {
    let target = [
        "--owner",
        "o",
        "--repo",
        "r",
        "--index",
        "2",
        "--dependency-type",
        "blocked_by",
        "--related-issue-number",
        "5",
        "--related-owner",
        "other",
        "--related-repo",
        "support",
    ];
    let p = params(
        &[&["issues", "dependency-add"][..], &target].concat(),
        "issue_write",
    );
    assert_eq!(
        p,
        json!({"method":"add_dependency","owner":"o","repo":"r","issue_number":2,"dependency_type":"blocked_by","related_issue_number":5,"related_owner":"other","related_repo":"support"})
    );
    let remove = [&["issues", "dependency-remove"][..], &target].concat();
    assert!(plan(&remove).unwrap_err().to_string().contains("--yes"));
    assert_eq!(
        params(&[&remove[..], &["--yes"]].concat(), "issue_write")["method"],
        "remove_dependency"
    );
}

#[test]
fn project_scopes_and_all_methods_map_correctly() {
    assert_eq!(
        params(&["projects", "list"], "project_read"),
        json!({"method":"list","state":"all","page":1,"per_page":30})
    );
    assert_eq!(
        params(&["projects", "list", "--org", "team"], "project_read")["org"],
        "team"
    );
    assert_eq!(
        params(
            &["projects", "list", "--owner", "o", "--repo", "r"],
            "project_read"
        )["owner"],
        "o"
    );
    for args in [
        vec!["projects", "list", "--owner", "o"],
        vec!["projects", "list", "--repo", "r"],
        vec![
            "projects", "list", "--owner", "o", "--repo", "r", "--org", "team",
        ],
        vec!["projects", "list", "--page", "0"],
    ] {
        assert!(plan(&args).is_err());
    }
    for (cmd, method, extra, tool) in [
        ("get", "get", vec![], "project_read"),
        ("columns", "list_columns", vec![], "project_read"),
        (
            "column-issues",
            "list_column_issues",
            vec!["--column-id", "8"],
            "project_read",
        ),
        (
            "update",
            "update",
            vec!["--state", "closed"],
            "project_write",
        ),
        ("delete", "delete", vec!["--yes"], "project_write"),
        (
            "column-create",
            "create_column",
            vec!["--title", "Todo"],
            "project_write",
        ),
        (
            "column-update",
            "update_column",
            vec!["--column-id", "8", "--title", "Done"],
            "project_write",
        ),
        (
            "column-delete",
            "delete_column",
            vec!["--column-id", "8", "--yes"],
            "project_write",
        ),
        (
            "issue-add",
            "add_issue",
            vec!["--issue-id", "99", "--column-id", "8"],
            "project_write",
        ),
        (
            "issue-remove",
            "remove_issue",
            vec!["--issue-id", "99", "--yes"],
            "project_write",
        ),
    ] {
        let p = params(
            &[
                &["projects", cmd, "--project-id", "7", "--org", "team"][..],
                &extra,
            ]
            .concat(),
            tool,
        );
        assert_eq!(p["method"], method);
        assert_eq!(p["project_id"], 7);
        assert_eq!(p["org"], "team");
        if extra.contains(&"--column-id") {
            assert_eq!(p["column_id"], 8);
        }
        if extra.contains(&"--issue-id") {
            assert_eq!(p["issue_id"], 99);
            assert!(p.get("issue_number").is_none());
        }
    }
    assert_eq!(
        params(
            &[
                "projects",
                "create",
                "--title",
                "Board",
                "--description",
                "",
                "--template-type",
                "basic_kanban"
            ],
            "project_write"
        ),
        json!({"method":"create","title":"Board","description":"","template_type":"basic_kanban"})
    );
    for (command, extra) in [
        ("delete", vec![]),
        ("column-delete", vec!["--column-id", "8"]),
        ("issue-remove", vec!["--issue-id", "99"]),
    ] {
        let error =
            plan(&[&["projects", command, "--project-id", "7"][..], &extra].concat()).unwrap_err();
        assert!(error.to_string().contains("--yes"));
    }
    assert!(plan(&["projects", "update", "--project-id", "7"]).is_err());
}
