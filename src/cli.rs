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
    /// 检查 gitea-cli 与底层 Gitea MCP 配置是否可用
    Doctor,
    /// 查看 gitea-cli 封装后的 MCP 工具能力
    Tools(ToolsCommand),
    /// 读取当前认证用户信息
    Me,
    /// 查询当前用户可访问的组织
    Orgs(OrgsCommand),
    /// 查询仓库列表、分支和文件树
    Repos(ReposCommand),
    /// 查询仓库 release 列表、最新版本和单个 release
    Releases(ReleasesCommand),
    /// 查询仓库 tag 列表和单个 tag 详情
    Tags(TagsCommand),
    /// 查询提交历史和单个 commit 详情
    Commits(CommitsCommand),
    /// 查询 issue 列表、详情、评论与跨仓库搜索
    Issues(IssuesCommand),
    /// 查询 pull request 列表、详情和 diff
    Pulls(PullsCommand),
    /// 查询 Actions workflow、run、job 与日志预览
    Actions(ActionsCommand),
    /// 从 Gitea URL 解析 owner、repo、issue 或 pull 坐标
    Resolve(ResolveCommand),
    /// 直接调用底层 MCP 工具，作为原始逃生口
    Mcp(McpCommand),
}

#[derive(Debug, Clone, Args)]
pub struct ToolsCommand {
    #[command(subcommand)]
    pub command: ToolsSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ToolsSubcommand {
    /// 列出 gitea-cli 已封装的工具能力
    List,
}

#[derive(Debug, Clone, Args)]
pub struct OrgsCommand {
    #[command(subcommand)]
    pub command: OrgsSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum OrgsSubcommand {
    /// 列出当前用户可访问的组织
    List(PageArgs),
}

#[derive(Debug, Clone, Args)]
pub struct ReposCommand {
    #[command(subcommand)]
    pub command: ReposSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ReposSubcommand {
    /// 列出当前用户仓库，或按组织列出仓库
    List(RepoListArgs),
    /// 列出指定仓库的分支
    Branches(RepoTargetWithPageArgs),
    /// 读取指定仓库在某个 ref 下的文件树
    Tree(RepoTreeArgs),
}

#[derive(Debug, Clone, Args)]
pub struct ReleasesCommand {
    #[command(subcommand)]
    pub command: ReleasesSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ReleasesSubcommand {
    /// 列出指定仓库的 release 列表
    List(RepoTargetWithPageArgs),
    /// 读取指定仓库的最新 release
    Latest(RepoTargetArgs),
    /// 按 ID 读取单个 release 详情
    Get(ReleaseTargetArgs),
}

#[derive(Debug, Clone, Args)]
pub struct TagsCommand {
    #[command(subcommand)]
    pub command: TagsSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum TagsSubcommand {
    /// 列出指定仓库的 tag 列表
    List(RepoTargetWithPageArgs),
    /// 按名称读取单个 tag 详情
    Get(TagTargetArgs),
}

#[derive(Debug, Clone, Args)]
pub struct CommitsCommand {
    #[command(subcommand)]
    pub command: CommitsSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum CommitsSubcommand {
    /// 列出指定仓库的提交历史
    List(CommitsListArgs),
    /// 按 SHA 读取单个 commit 详情
    Get(CommitTargetArgs),
}

#[derive(Debug, Clone, Args)]
pub struct IssuesCommand {
    #[command(subcommand)]
    pub command: IssuesSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum IssuesSubcommand {
    /// 列出仓库 issue 列表
    List(IssuesListArgs),
    /// 读取单个 issue 详情
    Get(IssueTargetArgs),
    /// 读取单个 issue 的评论列表
    Comments(IssueTargetArgs),
    /// 按关键词跨仓库搜索 issue 或 pull request
    Search(IssueSearchArgs),
}

#[derive(Debug, Clone, Args)]
pub struct PullsCommand {
    #[command(subcommand)]
    pub command: PullsSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum PullsSubcommand {
    /// 列出仓库 pull request 列表
    List(PullsListArgs),
    /// 读取单个 pull request 详情
    Get(PullTargetArgs),
    /// 读取单个 pull request 的 diff
    Diff(PullDiffArgs),
}

#[derive(Debug, Clone, Args)]
pub struct ActionsCommand {
    #[command(subcommand)]
    pub command: ActionsSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ActionsSubcommand {
    /// 列出指定仓库的 workflow
    Workflows(RepoTargetArgs),
    /// 列出指定仓库的 workflow runs
    Runs(ActionsRunsArgs),
    /// 列出仓库 jobs，或按 run 读取 jobs
    Jobs(ActionsJobsArgs),
    /// 预览某个 job 的日志尾部
    LogPreview(ActionsLogPreviewArgs),
}

#[derive(Debug, Clone, Args)]
pub struct ResolveCommand {
    #[command(subcommand)]
    pub command: ResolveSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ResolveSubcommand {
    /// 解析仓库 URL
    Repo(ResolveUrlArgs),
    /// 解析 issue URL
    Issue(ResolveUrlArgs),
    /// 解析 pull request URL
    Pull(ResolveUrlArgs),
}

#[derive(Debug, Clone, Args)]
pub struct ResolveUrlArgs {
    /// 要解析的 Gitea 页面 URL
    pub url: String,
}

#[derive(Debug, Clone, Args)]
pub struct McpCommand {
    #[command(subcommand)]
    pub command: McpSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum McpSubcommand {
    /// 直接调用底层 MCP 工具
    Call(McpCallArgs),
}

#[derive(Debug, Clone, Args)]
pub struct McpCallArgs {
    /// 底层 MCP 工具名
    pub tool_name: String,
    /// 传给底层 MCP 工具的 JSON 参数
    #[arg(long, default_value = "{}")]
    pub params: String,
}

#[derive(Debug, Clone, Args)]
pub struct PageArgs {
    /// 页码，从 1 开始
    #[arg(long, default_value_t = 1)]
    pub page: u32,
    /// 每页返回条数
    #[arg(long = "page-size", default_value_t = 30)]
    pub page_size: u32,
}

#[derive(Debug, Clone, Args)]
pub struct RepoTargetArgs {
    /// Gitea 仓库所属 owner 或组织
    #[arg(long)]
    pub owner: String,
    /// Gitea 仓库名
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
    /// 组织名；不传时列出当前用户自己的仓库
    #[arg(long)]
    pub owner: Option<String>,
    #[command(flatten)]
    pub page: PageArgs,
}

#[derive(Debug, Clone, Args)]
pub struct RepoTreeArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    /// 要读取的分支、tag 或 commit，默认 main
    #[arg(long = "ref", default_value = "main")]
    pub git_ref: String,
    /// 是否递归展开整个文件树
    #[arg(long)]
    pub recursive: bool,
    /// 页码，从 1 开始
    #[arg(long, default_value_t = 1)]
    pub page: u32,
    /// 每页返回条数
    #[arg(long = "page-size", default_value_t = 100)]
    pub page_size: u32,
}

#[derive(Debug, Clone, Args)]
pub struct ReleaseTargetArgs {
    /// Gitea 仓库所属 owner 或组织
    #[arg(long)]
    pub owner: String,
    /// Gitea 仓库名
    #[arg(long)]
    pub repo: String,
    /// Release ID
    #[arg(long)]
    pub id: u64,
}

#[derive(Debug, Clone, Args)]
pub struct TagTargetArgs {
    /// Gitea 仓库所属 owner 或组织
    #[arg(long)]
    pub owner: String,
    /// Gitea 仓库名
    #[arg(long)]
    pub repo: String,
    /// Tag 名称
    #[arg(long = "tag")]
    pub tag_name: String,
}

#[derive(Debug, Clone, Args)]
pub struct CommitTargetArgs {
    /// Gitea 仓库所属 owner 或组织
    #[arg(long)]
    pub owner: String,
    /// Gitea 仓库名
    #[arg(long)]
    pub repo: String,
    /// Commit SHA
    #[arg(long)]
    pub sha: String,
}

#[derive(Debug, Clone, Args)]
pub struct CommitsListArgs {
    /// Gitea 仓库所属 owner 或组织
    #[arg(long)]
    pub owner: String,
    /// Gitea 仓库名
    #[arg(long)]
    pub repo: String,
    /// 起始分支、tag 或 commit SHA
    #[arg(long)]
    pub sha: Option<String>,
    /// 只返回包含指定文件或目录路径的提交
    #[arg(long)]
    pub path: Option<String>,
    #[command(flatten)]
    pub page: PageArgs,
}

#[derive(Debug, Clone, Args)]
pub struct IssueTargetArgs {
    /// Gitea 仓库所属 owner 或组织
    #[arg(long)]
    pub owner: String,
    /// Gitea 仓库名
    #[arg(long)]
    pub repo: String,
    /// Issue 编号
    #[arg(long)]
    pub index: u64,
}

#[derive(Debug, Clone, Args)]
pub struct IssuesListArgs {
    /// Gitea 仓库所属 owner 或组织
    #[arg(long)]
    pub owner: String,
    /// Gitea 仓库名
    #[arg(long)]
    pub repo: String,
    /// Issue 状态过滤，默认 open
    #[arg(long, default_value = "open")]
    pub state: String,
    /// 按标签过滤，可重复传入
    #[arg(long)]
    pub labels: Vec<String>,
    /// 仅返回此时间之后更新的 issue
    #[arg(long)]
    pub since: Option<String>,
    /// 仅返回此时间之前更新的 issue
    #[arg(long)]
    pub before: Option<String>,
    #[command(flatten)]
    pub page: PageArgs,
}

#[derive(Debug, Clone, Args)]
pub struct IssueSearchArgs {
    /// 搜索关键词
    #[arg(long)]
    pub query: String,
    /// 限定 owner 或组织
    #[arg(long)]
    pub owner: Option<String>,
    /// 按状态过滤
    #[arg(long)]
    pub state: Option<String>,
    /// 按标签过滤，可重复传入
    #[arg(long)]
    pub labels: Vec<String>,
    /// 页码，从 1 开始
    #[arg(long, default_value_t = 1)]
    pub page: u32,
    /// 每页返回条数
    #[arg(long = "page-size", default_value_t = 30)]
    pub page_size: u32,
}

#[derive(Debug, Clone, Args)]
pub struct PullTargetArgs {
    /// Gitea 仓库所属 owner 或组织
    #[arg(long)]
    pub owner: String,
    /// Gitea 仓库名
    #[arg(long)]
    pub repo: String,
    /// Pull request 编号
    #[arg(long)]
    pub index: u64,
}

#[derive(Debug, Clone, Args)]
pub struct PullDiffArgs {
    #[command(flatten)]
    pub target: PullTargetArgs,
    /// 是否包含二进制文件变更
    #[arg(long)]
    pub binary: bool,
}

#[derive(Debug, Clone, Args)]
pub struct PullsListArgs {
    /// Gitea 仓库所属 owner 或组织
    #[arg(long)]
    pub owner: String,
    /// Gitea 仓库名
    #[arg(long)]
    pub repo: String,
    /// Pull request 状态过滤，默认 open
    #[arg(long, default_value = "open")]
    pub state: String,
    /// 排序方式
    #[arg(long)]
    pub sort: Option<String>,
    /// 里程碑编号过滤
    #[arg(long)]
    pub milestone: Option<u64>,
    #[command(flatten)]
    pub page: PageArgs,
}

#[derive(Debug, Clone, Args)]
pub struct ActionsRunsArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    /// 按运行状态过滤
    #[arg(long)]
    pub status: Option<String>,
    #[command(flatten)]
    pub page: PageArgs,
}

#[derive(Debug, Clone, Args)]
pub struct ActionsJobsArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    /// 指定 workflow run ID；不传则列出仓库 jobs
    #[arg(long = "run-id")]
    pub run_id: Option<u64>,
    /// 按 job 状态过滤
    #[arg(long)]
    pub status: Option<String>,
    #[command(flatten)]
    pub page: PageArgs,
}

#[derive(Debug, Clone, Args)]
pub struct ActionsLogPreviewArgs {
    /// Gitea 仓库所属 owner 或组织
    #[arg(long)]
    pub owner: String,
    /// Gitea 仓库名
    #[arg(long)]
    pub repo: String,
    /// Job ID
    #[arg(long = "job-id")]
    pub job_id: u64,
    /// 只返回日志尾部的行数
    #[arg(long = "tail-lines")]
    pub tail_lines: Option<u64>,
    /// 返回日志的最大字节数
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
