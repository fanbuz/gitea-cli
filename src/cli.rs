use std::fs;

use anyhow::{Context, Result, anyhow, bail};
use clap::{Args, Parser, Subcommand};
use regex::Regex;
use serde_json::{Map, Value, json};

#[derive(Debug, Clone, PartialEq)]
pub enum PlannedCommand {
    Doctor,
    ToolsList,
    ToolCall { tool: String, params: Value },
    Resolve { result: Value },
}

impl PlannedCommand {
    pub fn tool_call(tool: impl Into<String>, params: Value) -> Self {
        Self::ToolCall {
            tool: tool.into(),
            params,
        }
    }

    pub fn resolve(result: Value) -> Self {
        Self::Resolve { result }
    }
}

#[derive(Debug, Clone, Parser)]
#[command(
    name = "gitea-cli",
    version,
    about = "CLI wrapper around a configured Gitea MCP server"
)]
pub struct Cli {
    #[arg(long, global = true, help = "输出紧凑 JSON")]
    pub json: bool,
    #[command(subcommand)]
    pub command: Command,
}

impl Cli {
    pub fn try_parse_from<I, T>(itr: I) -> clap::error::Result<Self>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        <Self as Parser>::try_parse_from(itr)
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    Doctor,
    Tools(ToolsCommand),
    Me,
    Orgs(OrgsCommand),
    Repos(ReposCommand),
    Releases(ReleasesCommand),
    Tags(TagsCommand),
    Commits(CommitsCommand),
    Issues(IssuesCommand),
    Pulls(PullsCommand),
    Actions(ActionsCommand),
    Resolve(ResolveCommand),
    Mcp(McpCommand),
}

#[derive(Debug, Clone, Args)]
pub struct ToolsCommand {
    #[command(subcommand)]
    pub command: ToolsSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ToolsSubcommand {
    List,
}

#[derive(Debug, Clone, Args)]
pub struct OrgsCommand {
    #[command(subcommand)]
    pub command: OrgsSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum OrgsSubcommand {
    List(PageArgs),
}

#[derive(Debug, Clone, Args)]
pub struct ReposCommand {
    #[command(subcommand)]
    pub command: ReposSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ReposSubcommand {
    List(RepoListArgs),
    Branches(RepoTargetWithPageArgs),
    Tree(RepoTreeArgs),
}

#[derive(Debug, Clone, Args)]
pub struct ReleasesCommand {
    #[command(subcommand)]
    pub command: ReleasesSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ReleasesSubcommand {
    List(RepoTargetWithPageArgs),
    Latest(RepoTargetArgs),
    Get(ReleaseTargetArgs),
}

#[derive(Debug, Clone, Args)]
pub struct TagsCommand {
    #[command(subcommand)]
    pub command: TagsSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum TagsSubcommand {
    List(RepoTargetWithPageArgs),
    Get(TagTargetArgs),
}

#[derive(Debug, Clone, Args)]
pub struct CommitsCommand {
    #[command(subcommand)]
    pub command: CommitsSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum CommitsSubcommand {
    List(CommitsListArgs),
    Get(CommitTargetArgs),
}

#[derive(Debug, Clone, Args)]
pub struct IssuesCommand {
    #[command(subcommand)]
    pub command: IssuesSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum IssuesSubcommand {
    List(IssuesListArgs),
    Get(IssueTargetArgs),
    Comments(IssueTargetArgs),
    Search(IssueSearchArgs),
}

#[derive(Debug, Clone, Args)]
pub struct PullsCommand {
    #[command(subcommand)]
    pub command: PullsSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum PullsSubcommand {
    List(PullsListArgs),
    Get(PullTargetArgs),
    Diff(PullDiffArgs),
}

#[derive(Debug, Clone, Args)]
pub struct ActionsCommand {
    #[command(subcommand)]
    pub command: ActionsSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ActionsSubcommand {
    Workflows(RepoTargetArgs),
    Runs(ActionsRunsArgs),
    Jobs(ActionsJobsArgs),
    LogPreview(ActionsLogPreviewArgs),
}

#[derive(Debug, Clone, Args)]
pub struct ResolveCommand {
    #[command(subcommand)]
    pub command: ResolveSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ResolveSubcommand {
    Repo(ResolveUrlArgs),
    Issue(ResolveUrlArgs),
    Pull(ResolveUrlArgs),
}

#[derive(Debug, Clone, Args)]
pub struct ResolveUrlArgs {
    pub url: String,
}

#[derive(Debug, Clone, Args)]
pub struct McpCommand {
    #[command(subcommand)]
    pub command: McpSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum McpSubcommand {
    Call(McpCallArgs),
}

#[derive(Debug, Clone, Args)]
pub struct McpCallArgs {
    pub tool_name: String,
    #[arg(long, default_value = "{}")]
    pub params: String,
}

#[derive(Debug, Clone, Args)]
pub struct PageArgs {
    #[arg(long, default_value_t = 1)]
    pub page: u32,
    #[arg(long = "page-size", default_value_t = 30)]
    pub page_size: u32,
}

#[derive(Debug, Clone, Args)]
pub struct RepoTargetArgs {
    #[arg(long)]
    pub owner: String,
    #[arg(long)]
    pub repo: String,
}

#[derive(Debug, Clone, Args)]
pub struct RepoTargetWithPageArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    #[command(flatten)]
    pub page: PageArgs,
}

#[derive(Debug, Clone, Args)]
pub struct RepoListArgs {
    #[arg(long)]
    pub owner: Option<String>,
    #[command(flatten)]
    pub page: PageArgs,
}

#[derive(Debug, Clone, Args)]
pub struct RepoTreeArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    #[arg(long = "ref", default_value = "main")]
    pub git_ref: String,
    #[arg(long)]
    pub recursive: bool,
    #[arg(long, default_value_t = 1)]
    pub page: u32,
    #[arg(long = "page-size", default_value_t = 100)]
    pub page_size: u32,
}

#[derive(Debug, Clone, Args)]
pub struct ReleaseTargetArgs {
    #[arg(long)]
    pub owner: String,
    #[arg(long)]
    pub repo: String,
    #[arg(long)]
    pub id: u64,
}

#[derive(Debug, Clone, Args)]
pub struct TagTargetArgs {
    #[arg(long)]
    pub owner: String,
    #[arg(long)]
    pub repo: String,
    #[arg(long = "tag")]
    pub tag_name: String,
}

#[derive(Debug, Clone, Args)]
pub struct CommitTargetArgs {
    #[arg(long)]
    pub owner: String,
    #[arg(long)]
    pub repo: String,
    #[arg(long)]
    pub sha: String,
}

#[derive(Debug, Clone, Args)]
pub struct CommitsListArgs {
    #[arg(long)]
    pub owner: String,
    #[arg(long)]
    pub repo: String,
    #[arg(long)]
    pub sha: Option<String>,
    #[arg(long)]
    pub path: Option<String>,
    #[command(flatten)]
    pub page: PageArgs,
}

#[derive(Debug, Clone, Args)]
pub struct IssueTargetArgs {
    #[arg(long)]
    pub owner: String,
    #[arg(long)]
    pub repo: String,
    #[arg(long)]
    pub index: u64,
}

#[derive(Debug, Clone, Args)]
pub struct IssuesListArgs {
    #[arg(long)]
    pub owner: String,
    #[arg(long)]
    pub repo: String,
    #[arg(long, default_value = "open")]
    pub state: String,
    #[arg(long)]
    pub labels: Vec<String>,
    #[arg(long)]
    pub since: Option<String>,
    #[arg(long)]
    pub before: Option<String>,
    #[command(flatten)]
    pub page: PageArgs,
}

#[derive(Debug, Clone, Args)]
pub struct IssueSearchArgs {
    #[arg(long)]
    pub query: String,
    #[arg(long)]
    pub owner: Option<String>,
    #[arg(long)]
    pub state: Option<String>,
    #[arg(long)]
    pub labels: Vec<String>,
    #[arg(long, default_value_t = 1)]
    pub page: u32,
    #[arg(long = "page-size", default_value_t = 30)]
    pub page_size: u32,
}

#[derive(Debug, Clone, Args)]
pub struct PullTargetArgs {
    #[arg(long)]
    pub owner: String,
    #[arg(long)]
    pub repo: String,
    #[arg(long)]
    pub index: u64,
}

#[derive(Debug, Clone, Args)]
pub struct PullDiffArgs {
    #[command(flatten)]
    pub target: PullTargetArgs,
    #[arg(long)]
    pub binary: bool,
}

#[derive(Debug, Clone, Args)]
pub struct PullsListArgs {
    #[arg(long)]
    pub owner: String,
    #[arg(long)]
    pub repo: String,
    #[arg(long, default_value = "open")]
    pub state: String,
    #[arg(long)]
    pub sort: Option<String>,
    #[arg(long)]
    pub milestone: Option<u64>,
    #[command(flatten)]
    pub page: PageArgs,
}

#[derive(Debug, Clone, Args)]
pub struct ActionsRunsArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    #[arg(long)]
    pub status: Option<String>,
    #[command(flatten)]
    pub page: PageArgs,
}

#[derive(Debug, Clone, Args)]
pub struct ActionsJobsArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    #[arg(long = "run-id")]
    pub run_id: Option<u64>,
    #[arg(long)]
    pub status: Option<String>,
    #[command(flatten)]
    pub page: PageArgs,
}

#[derive(Debug, Clone, Args)]
pub struct ActionsLogPreviewArgs {
    #[arg(long)]
    pub owner: String,
    #[arg(long)]
    pub repo: String,
    #[arg(long = "job-id")]
    pub job_id: u64,
    #[arg(long = "tail-lines")]
    pub tail_lines: Option<u64>,
    #[arg(long = "max-bytes")]
    pub max_bytes: Option<u64>,
}

pub fn plan_command(cli: &Cli) -> Result<PlannedCommand> {
    match &cli.command {
        Command::Doctor => Ok(PlannedCommand::Doctor),
        Command::Tools(command) => match &command.command {
            ToolsSubcommand::List => Ok(PlannedCommand::ToolsList),
        },
        Command::Me => Ok(PlannedCommand::tool_call("get_me", json!({}))),
        Command::Orgs(command) => match &command.command {
            OrgsSubcommand::List(args) => Ok(PlannedCommand::tool_call(
                "get_user_orgs",
                json!({
                    "page": args.page,
                    "perPage": args.page_size
                }),
            )),
        },
        Command::Repos(command) => plan_repos(command),
        Command::Releases(command) => plan_releases(command),
        Command::Tags(command) => plan_tags(command),
        Command::Commits(command) => plan_commits(command),
        Command::Issues(command) => plan_issues(command),
        Command::Pulls(command) => plan_pulls(command),
        Command::Actions(command) => plan_actions(command),
        Command::Resolve(command) => plan_resolve(command),
        Command::Mcp(command) => plan_mcp(command),
    }
}

fn plan_repos(command: &ReposCommand) -> Result<PlannedCommand> {
    match &command.command {
        ReposSubcommand::List(args) => {
            if let Some(owner) = &args.owner {
                Ok(PlannedCommand::tool_call(
                    "list_org_repos",
                    json!({
                        "org": owner,
                        "page": args.page.page,
                        "pageSize": args.page.page_size
                    }),
                ))
            } else {
                Ok(PlannedCommand::tool_call(
                    "list_my_repos",
                    json!({
                        "page": args.page.page,
                        "perPage": args.page.page_size
                    }),
                ))
            }
        }
        ReposSubcommand::Branches(args) => Ok(PlannedCommand::tool_call(
            "list_branches",
            json!({
                "owner": args.target.owner,
                "repo": args.target.repo,
                "page": args.page.page,
                "perPage": args.page.page_size
            }),
        )),
        ReposSubcommand::Tree(args) => Ok(PlannedCommand::tool_call(
            "get_repository_tree",
            json!({
                "owner": args.target.owner,
                "repo": args.target.repo,
                "tree_sha": args.git_ref,
                "recursive": args.recursive,
                "page": args.page,
                "perPage": args.page_size
            }),
        )),
    }
}

fn plan_issues(command: &IssuesCommand) -> Result<PlannedCommand> {
    match &command.command {
        IssuesSubcommand::List(args) => {
            let mut params = Map::new();
            params.insert("owner".to_string(), json!(args.owner));
            params.insert("repo".to_string(), json!(args.repo));
            params.insert("state".to_string(), json!(args.state));
            params.insert("page".to_string(), json!(args.page.page));
            params.insert("perPage".to_string(), json!(args.page.page_size));
            if !args.labels.is_empty() {
                params.insert("labels".to_string(), json!(args.labels));
            }
            insert_optional_string(&mut params, "since", args.since.as_deref());
            insert_optional_string(&mut params, "before", args.before.as_deref());
            Ok(PlannedCommand::tool_call(
                "list_issues",
                Value::Object(params),
            ))
        }
        IssuesSubcommand::Get(args) => Ok(PlannedCommand::tool_call(
            "issue_read",
            json!({
                "owner": args.owner,
                "repo": args.repo,
                "index": args.index,
                "method": "get"
            }),
        )),
        IssuesSubcommand::Comments(args) => Ok(PlannedCommand::tool_call(
            "issue_read",
            json!({
                "owner": args.owner,
                "repo": args.repo,
                "index": args.index,
                "method": "get_comments"
            }),
        )),
        IssuesSubcommand::Search(args) => {
            let mut params = Map::new();
            params.insert("query".to_string(), json!(args.query));
            params.insert("type".to_string(), json!("issues"));
            params.insert("page".to_string(), json!(args.page));
            params.insert("perPage".to_string(), json!(args.page_size));
            insert_optional_string(&mut params, "owner", args.owner.as_deref());
            insert_optional_string(&mut params, "state", args.state.as_deref());
            if !args.labels.is_empty() {
                params.insert("labels".to_string(), json!(args.labels.join(",")));
            }
            Ok(PlannedCommand::tool_call(
                "search_issues",
                Value::Object(params),
            ))
        }
    }
}

fn plan_releases(command: &ReleasesCommand) -> Result<PlannedCommand> {
    match &command.command {
        ReleasesSubcommand::List(args) => Ok(PlannedCommand::tool_call(
            "list_releases",
            json!({
                "owner": args.target.owner,
                "repo": args.target.repo,
                "page": args.page.page,
                "perPage": args.page.page_size
            }),
        )),
        ReleasesSubcommand::Latest(args) => Ok(PlannedCommand::tool_call(
            "get_latest_release",
            json!({
                "owner": args.owner,
                "repo": args.repo
            }),
        )),
        ReleasesSubcommand::Get(args) => Ok(PlannedCommand::tool_call(
            "get_release",
            json!({
                "owner": args.owner,
                "repo": args.repo,
                "id": args.id
            }),
        )),
    }
}

fn plan_tags(command: &TagsCommand) -> Result<PlannedCommand> {
    match &command.command {
        TagsSubcommand::List(args) => Ok(PlannedCommand::tool_call(
            "list_tags",
            json!({
                "owner": args.target.owner,
                "repo": args.target.repo,
                "page": args.page.page,
                "perPage": args.page.page_size
            }),
        )),
        TagsSubcommand::Get(args) => Ok(PlannedCommand::tool_call(
            "get_tag",
            json!({
                "owner": args.owner,
                "repo": args.repo,
                "tag_name": args.tag_name
            }),
        )),
    }
}

fn plan_commits(command: &CommitsCommand) -> Result<PlannedCommand> {
    match &command.command {
        CommitsSubcommand::List(args) => {
            let mut params = Map::new();
            params.insert("owner".to_string(), json!(args.owner));
            params.insert("repo".to_string(), json!(args.repo));
            params.insert("page".to_string(), json!(args.page.page));
            params.insert("perPage".to_string(), json!(args.page.page_size));
            insert_optional_string(&mut params, "sha", args.sha.as_deref());
            insert_optional_string(&mut params, "path", args.path.as_deref());
            Ok(PlannedCommand::tool_call(
                "list_commits",
                Value::Object(params),
            ))
        }
        CommitsSubcommand::Get(args) => Ok(PlannedCommand::tool_call(
            "get_commit",
            json!({
                "owner": args.owner,
                "repo": args.repo,
                "sha": args.sha
            }),
        )),
    }
}

fn plan_pulls(command: &PullsCommand) -> Result<PlannedCommand> {
    match &command.command {
        PullsSubcommand::List(args) => {
            let mut params = Map::new();
            params.insert("owner".to_string(), json!(args.owner));
            params.insert("repo".to_string(), json!(args.repo));
            params.insert("state".to_string(), json!(args.state));
            params.insert("page".to_string(), json!(args.page.page));
            params.insert("perPage".to_string(), json!(args.page.page_size));
            if let Some(sort) = &args.sort {
                params.insert("sort".to_string(), json!(sort));
            }
            if let Some(milestone) = args.milestone {
                params.insert("milestone".to_string(), json!(milestone));
            }
            Ok(PlannedCommand::tool_call(
                "list_pull_requests",
                Value::Object(params),
            ))
        }
        PullsSubcommand::Get(args) => Ok(PlannedCommand::tool_call(
            "pull_request_read",
            json!({
                "owner": args.owner,
                "repo": args.repo,
                "index": args.index,
                "method": "get"
            }),
        )),
        PullsSubcommand::Diff(args) => Ok(PlannedCommand::tool_call(
            "pull_request_read",
            json!({
                "owner": args.target.owner,
                "repo": args.target.repo,
                "index": args.target.index,
                "method": "get_diff",
                "binary": args.binary
            }),
        )),
    }
}

fn plan_actions(command: &ActionsCommand) -> Result<PlannedCommand> {
    match &command.command {
        ActionsSubcommand::Workflows(args) => Ok(PlannedCommand::tool_call(
            "actions_run_read",
            json!({
                "method": "list_workflows",
                "owner": args.owner,
                "repo": args.repo
            }),
        )),
        ActionsSubcommand::Runs(args) => {
            let mut params = Map::new();
            params.insert("method".to_string(), json!("list_runs"));
            params.insert("owner".to_string(), json!(args.target.owner));
            params.insert("repo".to_string(), json!(args.target.repo));
            params.insert("page".to_string(), json!(args.page.page));
            params.insert("perPage".to_string(), json!(args.page.page_size));
            insert_optional_string(&mut params, "status", args.status.as_deref());
            Ok(PlannedCommand::tool_call(
                "actions_run_read",
                Value::Object(params),
            ))
        }
        ActionsSubcommand::Jobs(args) => {
            let mut params = Map::new();
            params.insert(
                "method".to_string(),
                json!(if args.run_id.is_some() {
                    "list_run_jobs"
                } else {
                    "list_jobs"
                }),
            );
            params.insert("owner".to_string(), json!(args.target.owner));
            params.insert("repo".to_string(), json!(args.target.repo));
            params.insert("page".to_string(), json!(args.page.page));
            params.insert("perPage".to_string(), json!(args.page.page_size));
            if let Some(run_id) = args.run_id {
                params.insert("run_id".to_string(), json!(run_id));
            }
            insert_optional_string(&mut params, "status", args.status.as_deref());
            Ok(PlannedCommand::tool_call(
                "actions_run_read",
                Value::Object(params),
            ))
        }
        ActionsSubcommand::LogPreview(args) => {
            let mut params = Map::new();
            params.insert("method".to_string(), json!("get_job_log_preview"));
            params.insert("owner".to_string(), json!(args.owner));
            params.insert("repo".to_string(), json!(args.repo));
            params.insert("job_id".to_string(), json!(args.job_id));
            if let Some(tail_lines) = args.tail_lines {
                params.insert("tail_lines".to_string(), json!(tail_lines));
            }
            if let Some(max_bytes) = args.max_bytes {
                params.insert("max_bytes".to_string(), json!(max_bytes));
            }
            Ok(PlannedCommand::tool_call(
                "actions_run_read",
                Value::Object(params),
            ))
        }
    }
}

fn plan_resolve(command: &ResolveCommand) -> Result<PlannedCommand> {
    match &command.command {
        ResolveSubcommand::Repo(args) => Ok(PlannedCommand::resolve(resolve_repo_url(&args.url)?)),
        ResolveSubcommand::Issue(args) => {
            Ok(PlannedCommand::resolve(resolve_issue_url(&args.url)?))
        }
        ResolveSubcommand::Pull(args) => Ok(PlannedCommand::resolve(resolve_pull_url(&args.url)?)),
    }
}

fn plan_mcp(command: &McpCommand) -> Result<PlannedCommand> {
    match &command.command {
        McpSubcommand::Call(args) => Ok(PlannedCommand::tool_call(
            &args.tool_name,
            parse_json_input(&args.params)?,
        )),
    }
}

fn insert_optional_string(target: &mut Map<String, Value>, key: &str, value: Option<&str>) {
    if let Some(value) = value {
        target.insert(key.to_string(), json!(value));
    }
}

fn parse_json_input(raw: &str) -> Result<Value> {
    let source = if let Some(path) = raw.strip_prefix('@') {
        fs::read_to_string(path).with_context(|| format!("读取 JSON 参数文件失败: {path}"))?
    } else {
        raw.to_string()
    };

    serde_json::from_str(&source).with_context(|| format!("解析 JSON 参数失败: {source}"))
}

fn resolve_repo_url(url: &str) -> Result<Value> {
    let regex = Regex::new(r#"https?://[^/]+/([^/]+)/([^/]+)/?$"#).unwrap();
    let captures = regex
        .captures(url)
        .ok_or_else(|| anyhow!("无法从 URL 解析仓库坐标: {url}"))?;
    let owner = captures.get(1).unwrap().as_str();
    let repo = captures.get(2).unwrap().as_str().trim_end_matches(".git");
    if repo.is_empty() {
        bail!("URL 中缺少仓库名: {url}");
    }
    Ok(json!({
        "owner": owner,
        "repo": repo
    }))
}

fn resolve_issue_url(url: &str) -> Result<Value> {
    let regex = Regex::new(r#"https?://[^/]+/([^/]+)/([^/]+)/issues/(\d+)/?$"#).unwrap();
    let captures = regex
        .captures(url)
        .ok_or_else(|| anyhow!("无法从 URL 解析 issue 坐标: {url}"))?;
    Ok(json!({
        "owner": captures.get(1).unwrap().as_str(),
        "repo": captures.get(2).unwrap().as_str().trim_end_matches(".git"),
        "index": captures.get(3).unwrap().as_str().parse::<u64>()?
    }))
}

fn resolve_pull_url(url: &str) -> Result<Value> {
    let regex = Regex::new(r#"https?://[^/]+/([^/]+)/([^/]+)/pulls/(\d+)/?$"#).unwrap();
    let captures = regex
        .captures(url)
        .ok_or_else(|| anyhow!("无法从 URL 解析 pull 坐标: {url}"))?;
    Ok(json!({
        "owner": captures.get(1).unwrap().as_str(),
        "repo": captures.get(2).unwrap().as_str().trim_end_matches(".git"),
        "index": captures.get(3).unwrap().as_str().parse::<u64>()?
    }))
}
