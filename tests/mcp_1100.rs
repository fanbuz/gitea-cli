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

const REPO: &[&str] = &["--owner", "team", "--repo", "project"];

fn command(group: &str, subcommand: &str, args: &[&str], tool: &str) -> Value {
    params(&[&[group, subcommand], REPO, args].concat(), tool)
}

#[test]
fn new_command_help_is_consistent() {
    Cli::command().debug_assert();
    for (group, commands) in [
        ("commits", vec!["compare", "status", "status-create"]),
        ("issues", vec!["assignee-candidates"]),
        (
            "pulls",
            vec!["commits", "by-branches", "by-commit", "reviewer-candidates"],
        ),
        (
            "actions",
            vec![
                "rerun-job",
                "rerun-failed-jobs",
                "approve",
                "workflow-enable",
                "workflow-disable",
            ],
        ),
    ] {
        for subcommand in commands {
            let err = Cli::try_parse_from(["gitea-cli", group, subcommand, "--help"]).unwrap_err();
            assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
        }
    }
}

#[test]
fn single_commit_diff_is_opt_in_and_existing_request_is_unchanged() {
    let original = json!({"owner":"team", "repo":"project", "sha":"abc"});
    assert_eq!(
        command("commits", "get", &["--sha", "abc"], "get_commit"),
        original
    );
    let mut with_diff = original;
    with_diff["include_diff"] = json!(true);
    assert_eq!(
        command(
            "commits",
            "get",
            &["--sha", "abc", "--include-diff"],
            "get_commit"
        ),
        with_diff
    );
}

#[test]
fn compare_preserves_base_head_and_ref_paths() {
    assert_eq!(
        command(
            "commits",
            "compare",
            &["--base", "release/v1", "--head", "feature/中文"],
            "compare_commits"
        ),
        json!({"owner":"team", "repo":"project", "base":"release/v1", "head":"feature/中文"})
    );
    for reference in ["main", "v1.10.0", "abc123"] {
        assert_eq!(
            command(
                "commits",
                "status",
                &["--ref", reference],
                "get_commit_status"
            ),
            json!({"owner":"team", "repo":"project", "ref":reference})
        );
    }
}

#[test]
fn commit_status_all_states_and_optional_fields_match_server_contract() {
    for state in ["pending", "success", "error", "failure", "warning"] {
        assert_eq!(
            command(
                "commits",
                "status-create",
                &["--sha", "abc", "--state", state],
                "create_commit_status"
            ),
            json!({"owner":"team", "repo":"project", "sha":"abc", "state":state})
        );
    }
    assert_eq!(
        command(
            "commits",
            "status-create",
            &[
                "--sha",
                "abc",
                "--state",
                "success",
                "--context",
                "ci/build",
                "--description",
                "构建通过\n$HOME",
                "--target-url",
                "https://ci.example.com/1"
            ],
            "create_commit_status"
        ),
        json!({"owner":"team", "repo":"project", "sha":"abc", "state":"success", "context":"ci/build", "description":"构建通过\n$HOME", "target_url":"https://ci.example.com/1"})
    );
    assert!(
        plan(
            &[
                &["commits", "status-create"],
                REPO,
                &["--sha", "abc", "--state", "unknown"]
            ]
            .concat()
        )
        .is_err()
    );
}

#[test]
fn pr_commits_include_the_pull_number_and_pagination() {
    for (args, page, size) in [
        (vec!["--index", "12"], 1, 30),
        (
            vec!["--index", "12", "--page", "2", "--page-size", "7"],
            2,
            7,
        ),
    ] {
        assert_eq!(
            command("pulls", "commits", &args, "pull_request_read"),
            json!({"method":"get_commits", "owner":"team", "repo":"project", "pull_number":12, "page":page, "per_page":size})
        );
    }
}

#[test]
fn pr_lookup_uses_branches_or_sha_without_a_pull_number() {
    assert_eq!(
        command(
            "pulls",
            "by-branches",
            &["--base", "main", "--head", "feature/docs"],
            "pull_request_read"
        ),
        json!({"method":"get_by_branches", "owner":"team", "repo":"project", "base":"main", "head":"feature/docs"})
    );
    assert_eq!(
        command("pulls", "by-commit", &["--sha", "abc"], "pull_request_read"),
        json!({"method":"get_by_commit", "owner":"team", "repo":"project", "sha":"abc"})
    );
}

#[test]
fn candidates_are_repository_scoped_and_need_no_issue_or_pr_number() {
    for (group, name, tool, method) in [
        (
            "issues",
            "assignee-candidates",
            "issue_read",
            "get_assignee_candidates",
        ),
        (
            "pulls",
            "reviewer-candidates",
            "pull_request_read",
            "get_reviewer_candidates",
        ),
    ] {
        assert_eq!(
            command(group, name, &[], tool),
            json!({"method":method, "owner":"team", "repo":"project"})
        );
    }
}

#[test]
fn pr_creation_preserves_repeated_people_teams_and_milestone() {
    let base = [
        "--head",
        "feature/docs",
        "--base",
        "main",
        "--title",
        "文档",
    ];
    assert_eq!(
        command("pulls", "create", &base, "pull_request_write"),
        json!({"method":"create", "owner":"team", "repo":"project", "head":"feature/docs", "base":"main", "title":"文档"})
    );
    let extra = [
        "--assignee",
        "alice",
        "--assignee",
        "bob",
        "--milestone",
        "3",
        "--reviewer",
        "carol",
        "--reviewer",
        "dave",
        "--team-reviewer",
        "maintainers",
        "--team-reviewer",
        "docs",
        "--label-id",
        "4",
        "--draft",
    ];
    assert_eq!(
        command(
            "pulls",
            "create",
            &[&base[..], &extra].concat(),
            "pull_request_write"
        ),
        json!({"method":"create", "owner":"team", "repo":"project", "head":"feature/docs", "base":"main", "title":"文档", "assignees":["alice","bob"], "milestone":3, "reviewers":["carol","dave"], "team_reviewers":["maintainers","docs"], "labels":[4], "draft":true})
    );
    assert_eq!(
        command(
            "pulls",
            "create",
            &[&base[..], &["--assignee", "alice"]].concat(),
            "pull_request_write"
        )["assignees"],
        json!(["alice"])
    );
}

#[test]
fn actions_job_rerun_needs_both_run_and_job_ids() {
    assert_eq!(
        command(
            "actions",
            "rerun-job",
            &["--run-id", "123", "--job-id", "456"],
            "actions_run_write"
        ),
        json!({"method":"rerun_job", "owner":"team", "repo":"project", "run_id":123, "job_id":456})
    );
    for args in [vec![], vec!["--run-id", "123"], vec!["--job-id", "456"]] {
        assert!(plan(&[&["actions", "rerun-job"], REPO, &args].concat()).is_err());
    }
}

#[test]
fn actions_failed_jobs_and_approval_are_run_scoped() {
    for (name, method) in [
        ("rerun-failed-jobs", "rerun_failed_jobs"),
        ("approve", "approve_run"),
    ] {
        assert_eq!(
            command("actions", name, &["--run-id", "123"], "actions_run_write"),
            json!({"method":method, "owner":"team", "repo":"project", "run_id":123})
        );
        assert!(plan(&[&["actions", name], REPO].concat()).is_err());
    }
}

#[test]
fn workflow_toggles_accept_id_or_filename_without_run_id() {
    for (name, method) in [
        ("workflow-enable", "enable_workflow"),
        ("workflow-disable", "disable_workflow"),
    ] {
        for workflow in ["build.yml", "12"] {
            assert_eq!(
                command(
                    "actions",
                    name,
                    &["--workflow-id", workflow],
                    "actions_run_write"
                ),
                json!({"method":method, "owner":"team", "repo":"project", "workflow_id":workflow})
            );
        }
        assert!(plan(&[&["actions", name], REPO].concat()).is_err());
    }
}

#[test]
fn new_commit_and_pr_operations_reject_missing_selectors() {
    for (group, name, args) in [
        ("commits", "compare", vec!["--base", "main"]),
        ("commits", "compare", vec!["--head", "feature"]),
        ("commits", "status", vec![]),
        ("commits", "status-create", vec!["--sha", "abc"]),
        ("commits", "status-create", vec!["--state", "success"]),
        ("pulls", "commits", vec![]),
        ("pulls", "by-branches", vec!["--base", "main"]),
        ("pulls", "by-branches", vec!["--head", "feature"]),
        ("pulls", "by-commit", vec![]),
    ] {
        assert!(
            plan(&[&[group, name], REPO, &args].concat()).is_err(),
            "{group} {name} {args:?}"
        );
    }
}

#[test]
fn repo_label_exclusive_supports_creation_and_explicit_false_without_resetting_by_default() {
    let base = ["--name", "priority/high", "--color", "#ff0000"];
    let original = command("labels", "repo-create", &base, "label_write");
    assert!(original.get("exclusive").is_none());
    let mut expected = original;
    expected["exclusive"] = json!(true);
    assert_eq!(
        command(
            "labels",
            "repo-create",
            &[&base[..], &["--exclusive"]].concat(),
            "label_write"
        ),
        expected
    );
    assert!(
        command("labels", "repo-edit", &["--id", "6"], "label_write")
            .get("exclusive")
            .is_none()
    );
    for value in ["true", "false"] {
        assert_eq!(
            command(
                "labels",
                "repo-edit",
                &["--id", "6", "--exclusive", value],
                "label_write"
            ),
            json!({"method":"edit_repo_label", "owner":"team", "repo":"project", "id":6, "exclusive":value=="true"})
        );
    }
    assert!(
        plan(
            &[
                &["labels", "repo-edit"],
                REPO,
                &["--id", "6", "--exclusive", "invalid"]
            ]
            .concat()
        )
        .is_err()
    );
}
