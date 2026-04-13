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
    about = "CLI wrapper around a configured Gitea MCP server",
    after_help = concat!("当前版本: ", env!("CARGO_PKG_VERSION"))
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
    /// 查询和管理仓库 release
    Releases(ReleasesCommand),
    /// 查询和管理仓库 tag
    Tags(TagsCommand),
    /// 查询提交历史和单个 commit 详情
    Commits(CommitsCommand),
    /// 管理 issue、评论、labels 与 time tracking
    Issues(IssuesCommand),
    /// 管理仓库与组织 labels
    Labels(LabelsCommand),
    /// 管理仓库 milestones
    Milestones(MilestonesCommand),
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
    /// 创建 release
    Create(ReleaseCreateArgs),
    /// 删除 release
    Delete(ReleaseDeleteArgs),
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
    /// 创建 tag
    Create(TagCreateArgs),
    /// 删除 tag
    Delete(TagDeleteArgs),
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
    /// 创建 issue
    Create(IssueCreateArgs),
    /// 更新 issue
    Update(IssueUpdateArgs),
    /// 为 issue 添加评论
    CommentAdd(IssueCommentAddArgs),
    /// 编辑 issue 评论
    CommentEdit(IssueCommentEditArgs),
    /// 读取 issue 当前 labels
    Labels(IssueTargetArgs),
    /// 为 issue 添加 labels
    LabelsAdd(IssueLabelsArgs),
    /// 从 issue 删除一个 label
    LabelRemove(IssueLabelRemoveArgs),
    /// 用一组 labels 替换 issue 当前 labels
    LabelsReplace(IssueLabelsArgs),
    /// 清空 issue 当前 labels
    LabelsClear(IssueDangerousTargetArgs),
    /// 读取或写入 issue time tracking
    Time(IssueTimeCommand),
}

#[derive(Debug, Clone, Args)]
pub struct IssueTimeCommand {
    #[command(subcommand)]
    pub command: IssueTimeSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum IssueTimeSubcommand {
    /// 读取 issue time tracking 记录
    List(IssueTimeListArgs),
    /// 启动 issue stopwatch
    Start(IssueTargetArgs),
    /// 停止 issue stopwatch
    Stop(IssueTargetArgs),
    /// 清空 issue stopwatch
    ResetStopwatch(IssueDangerousTargetArgs),
    /// 为 issue 增加 tracked time
    Add(IssueTimeAddArgs),
    /// 删除一条 issue time 记录
    Delete(IssueTimeDeleteArgs),
}

#[derive(Debug, Clone, Args)]
pub struct PullsCommand {
    #[command(subcommand)]
    pub command: PullsSubcommand,
}

#[derive(Debug, Clone, Args)]
pub struct LabelsCommand {
    #[command(subcommand)]
    pub command: LabelsSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum LabelsSubcommand {
    /// 列出仓库 labels
    RepoList(RepoTargetWithPageArgs),
    /// 读取单个仓库 label
    RepoGet(RepoLabelTargetArgs),
    /// 创建仓库 label
    RepoCreate(RepoLabelCreateArgs),
    /// 编辑仓库 label
    RepoEdit(RepoLabelEditArgs),
    /// 删除仓库 label
    RepoDelete(RepoLabelDeleteArgs),
    /// 列出组织 labels
    OrgList(OrgLabelsListArgs),
    /// 创建组织 label
    OrgCreate(OrgLabelCreateArgs),
    /// 编辑组织 label
    OrgEdit(OrgLabelEditArgs),
    /// 删除组织 label
    OrgDelete(OrgLabelDeleteArgs),
}

#[derive(Debug, Clone, Args)]
pub struct MilestonesCommand {
    #[command(subcommand)]
    pub command: MilestonesSubcommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum MilestonesSubcommand {
    /// 列出仓库 milestones
    List(MilestoneListArgs),
    /// 读取单个 milestone
    Get(MilestoneTargetArgs),
    /// 创建 milestone
    Create(MilestoneCreateArgs),
    /// 编辑 milestone
    Edit(MilestoneEditArgs),
    /// 删除 milestone
    Delete(MilestoneDeleteArgs),
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
pub struct RepoLabelTargetArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    /// Label ID
    #[arg(long)]
    pub id: u64,
}

#[derive(Debug, Clone, Args)]
pub struct RepoLabelCreateArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    /// Label 名称
    #[arg(long)]
    pub name: String,
    /// Label 颜色，格式为 #RRGGBB
    #[arg(long)]
    pub color: String,
    /// Label 描述
    #[arg(long)]
    pub description: Option<String>,
    /// 创建归档 label
    #[arg(long)]
    pub archived: bool,
}

#[derive(Debug, Clone, Args)]
pub struct RepoLabelEditArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    /// Label ID
    #[arg(long)]
    pub id: u64,
    /// Label 名称
    #[arg(long)]
    pub name: Option<String>,
    /// Label 颜色，格式为 #RRGGBB
    #[arg(long)]
    pub color: Option<String>,
    /// Label 描述
    #[arg(long)]
    pub description: Option<String>,
    /// 是否归档，显式传 true 或 false
    #[arg(long)]
    pub archived: Option<bool>,
}

#[derive(Debug, Clone, Args)]
pub struct RepoLabelDeleteArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    /// Label ID
    #[arg(long)]
    pub id: u64,
    /// 确认执行危险操作
    #[arg(long)]
    pub yes: bool,
}

#[derive(Debug, Clone, Args)]
pub struct OrgLabelsListArgs {
    /// 组织名
    #[arg(long)]
    pub org: String,
    #[command(flatten)]
    pub page: PageArgs,
}

#[derive(Debug, Clone, Args)]
pub struct OrgLabelCreateArgs {
    /// 组织名
    #[arg(long)]
    pub org: String,
    /// Label 名称
    #[arg(long)]
    pub name: String,
    /// Label 颜色，格式为 #RRGGBB
    #[arg(long)]
    pub color: String,
    /// Label 描述
    #[arg(long)]
    pub description: Option<String>,
    /// 是否为 exclusive label
    #[arg(long)]
    pub exclusive: bool,
}

#[derive(Debug, Clone, Args)]
pub struct OrgLabelEditArgs {
    /// 组织名
    #[arg(long)]
    pub org: String,
    /// Label ID
    #[arg(long)]
    pub id: u64,
    /// Label 名称
    #[arg(long)]
    pub name: Option<String>,
    /// Label 颜色，格式为 #RRGGBB
    #[arg(long)]
    pub color: Option<String>,
    /// Label 描述
    #[arg(long)]
    pub description: Option<String>,
    /// 是否为 exclusive label，显式传 true 或 false
    #[arg(long)]
    pub exclusive: Option<bool>,
}

#[derive(Debug, Clone, Args)]
pub struct OrgLabelDeleteArgs {
    /// 组织名
    #[arg(long)]
    pub org: String,
    /// Label ID
    #[arg(long)]
    pub id: u64,
    /// 确认执行危险操作
    #[arg(long)]
    pub yes: bool,
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
pub struct ReleaseCreateArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    /// Release 对应的 tag 名称
    #[arg(long = "tag")]
    pub tag_name: String,
    /// Release 标题
    #[arg(long)]
    pub title: String,
    /// Release 指向的分支、tag 或 commit
    #[arg(long = "target")]
    pub target_ref: String,
    /// Release 正文
    #[arg(long)]
    pub body: Option<String>,
    /// 创建 draft release
    #[arg(long)]
    pub draft: bool,
    /// 创建 pre-release
    #[arg(long = "pre-release")]
    pub pre_release: bool,
}

#[derive(Debug, Clone, Args)]
pub struct ReleaseDeleteArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    /// Release ID
    #[arg(long)]
    pub id: u64,
    /// 确认执行危险操作
    #[arg(long)]
    pub yes: bool,
}

#[derive(Debug, Clone, Args)]
pub struct MilestoneTargetArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    /// Milestone ID
    #[arg(long)]
    pub id: u64,
}

#[derive(Debug, Clone, Args)]
pub struct MilestoneListArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    /// Milestone 状态
    #[arg(long)]
    pub state: Option<String>,
    /// Milestone 名称关键字
    #[arg(long)]
    pub name: Option<String>,
    #[command(flatten)]
    pub page: PageArgs,
}

#[derive(Debug, Clone, Args)]
pub struct MilestoneCreateArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    /// Milestone 标题
    #[arg(long)]
    pub title: String,
    /// Milestone 描述
    #[arg(long)]
    pub description: Option<String>,
    /// 截止时间，使用 ISO 8601
    #[arg(long = "due-on")]
    pub due_on: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct MilestoneEditArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    /// Milestone ID
    #[arg(long)]
    pub id: u64,
    /// Milestone 标题
    #[arg(long)]
    pub title: Option<String>,
    /// Milestone 描述
    #[arg(long)]
    pub description: Option<String>,
    /// 截止时间，使用 ISO 8601
    #[arg(long = "due-on")]
    pub due_on: Option<String>,
    /// Milestone 状态
    #[arg(long)]
    pub state: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct MilestoneDeleteArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    /// Milestone ID
    #[arg(long)]
    pub id: u64,
    /// 确认执行危险操作
    #[arg(long)]
    pub yes: bool,
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
pub struct TagCreateArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    /// Tag 名称
    #[arg(long = "tag")]
    pub tag_name: String,
    /// Tag 指向的分支、tag 或 commit
    #[arg(long = "target")]
    pub target_ref: Option<String>,
    /// Annotated tag 消息
    #[arg(long)]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct TagDeleteArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    /// Tag 名称
    #[arg(long = "tag")]
    pub tag_name: String,
    /// 确认执行危险操作
    #[arg(long)]
    pub yes: bool,
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
pub struct IssueCreateArgs {
    /// Gitea 仓库所属 owner 或组织
    #[arg(long)]
    pub owner: String,
    /// Gitea 仓库名
    #[arg(long)]
    pub repo: String,
    /// Issue 标题
    #[arg(long)]
    pub title: String,
    /// Issue 正文
    #[arg(long)]
    pub body: Option<String>,
    /// 指派用户，可重复传入
    #[arg(long = "assignee")]
    pub assignees: Vec<String>,
    /// Label ID，可重复传入
    #[arg(long = "label-id")]
    pub label_ids: Vec<u64>,
    /// Milestone 编号
    #[arg(long)]
    pub milestone: Option<u64>,
    /// 关联分支名
    #[arg(long = "ref")]
    pub git_ref: Option<String>,
    /// 截止时间，使用 ISO 8601
    #[arg(long)]
    pub deadline: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct IssueUpdateArgs {
    #[command(flatten)]
    pub target: IssueTargetArgs,
    /// Issue 标题
    #[arg(long)]
    pub title: Option<String>,
    /// Issue 正文
    #[arg(long)]
    pub body: Option<String>,
    /// Issue 状态
    #[arg(long)]
    pub state: Option<String>,
    /// 指派用户，可重复传入
    #[arg(long = "assignee")]
    pub assignees: Vec<String>,
    /// Label ID，可重复传入
    #[arg(long = "label-id")]
    pub label_ids: Vec<u64>,
    /// Milestone 编号
    #[arg(long)]
    pub milestone: Option<u64>,
    /// 关联分支名
    #[arg(long = "ref")]
    pub git_ref: Option<String>,
    /// 截止时间，使用 ISO 8601
    #[arg(long)]
    pub deadline: Option<String>,
    /// 清空截止时间
    #[arg(long)]
    pub remove_deadline: bool,
}

#[derive(Debug, Clone, Args)]
pub struct IssueCommentAddArgs {
    #[command(flatten)]
    pub target: IssueTargetArgs,
    /// 评论正文
    #[arg(long)]
    pub body: String,
}

#[derive(Debug, Clone, Args)]
pub struct IssueCommentEditArgs {
    #[command(flatten)]
    pub target: IssueTargetArgs,
    /// 评论 ID
    #[arg(long = "comment-id")]
    pub comment_id: u64,
    /// 评论正文
    #[arg(long)]
    pub body: String,
}

#[derive(Debug, Clone, Args)]
pub struct IssueLabelsArgs {
    #[command(flatten)]
    pub target: IssueTargetArgs,
    /// Label ID，可重复传入
    #[arg(long = "label-id")]
    pub label_ids: Vec<u64>,
}

#[derive(Debug, Clone, Args)]
pub struct IssueLabelRemoveArgs {
    #[command(flatten)]
    pub target: IssueTargetArgs,
    /// Label ID
    #[arg(long = "label-id")]
    pub label_id: u64,
    /// 确认执行危险操作
    #[arg(long)]
    pub yes: bool,
}

#[derive(Debug, Clone, Args)]
pub struct IssueDangerousTargetArgs {
    #[command(flatten)]
    pub target: IssueTargetArgs,
    /// 确认执行危险操作
    #[arg(long)]
    pub yes: bool,
}

#[derive(Debug, Clone, Args)]
pub struct IssueTimeListArgs {
    #[command(flatten)]
    pub target: IssueTargetArgs,
    #[command(flatten)]
    pub page: PageArgs,
}

#[derive(Debug, Clone, Args)]
pub struct IssueTimeAddArgs {
    #[command(flatten)]
    pub target: IssueTargetArgs,
    /// 要增加的秒数
    #[arg(long = "seconds")]
    pub seconds: u64,
}

#[derive(Debug, Clone, Args)]
pub struct IssueTimeDeleteArgs {
    #[command(flatten)]
    pub target: IssueTargetArgs,
    /// Time entry ID
    #[arg(long)]
    pub id: u64,
    /// 确认执行危险操作
    #[arg(long)]
    pub yes: bool,
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
        Command::Labels(command) => plan_labels(command),
        Command::Milestones(command) => plan_milestones(command),
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
        IssuesSubcommand::Create(args) => {
            let mut params = Map::new();
            params.insert("method".to_string(), json!("create"));
            params.insert("owner".to_string(), json!(args.owner));
            params.insert("repo".to_string(), json!(args.repo));
            params.insert("title".to_string(), json!(args.title));
            insert_optional_string(&mut params, "body", args.body.as_deref());
            insert_optional_string(&mut params, "ref", args.git_ref.as_deref());
            insert_optional_string(&mut params, "deadline", args.deadline.as_deref());
            insert_optional_u64(&mut params, "milestone", args.milestone);
            insert_optional_string_list(&mut params, "assignees", &args.assignees);
            insert_optional_u64_list(&mut params, "labels", &args.label_ids);
            Ok(PlannedCommand::tool_call(
                "issue_write",
                Value::Object(params),
            ))
        }
        IssuesSubcommand::Update(args) => {
            let mut params = Map::new();
            params.insert("method".to_string(), json!("update"));
            params.insert("owner".to_string(), json!(args.target.owner));
            params.insert("repo".to_string(), json!(args.target.repo));
            params.insert("index".to_string(), json!(args.target.index));
            insert_optional_string(&mut params, "title", args.title.as_deref());
            insert_optional_string(&mut params, "body", args.body.as_deref());
            insert_optional_string(&mut params, "state", args.state.as_deref());
            insert_optional_string(&mut params, "ref", args.git_ref.as_deref());
            insert_optional_string(&mut params, "deadline", args.deadline.as_deref());
            insert_optional_u64(&mut params, "milestone", args.milestone);
            insert_optional_string_list(&mut params, "assignees", &args.assignees);
            insert_optional_u64_list(&mut params, "labels", &args.label_ids);
            if args.remove_deadline {
                params.insert("remove_deadline".to_string(), json!(true));
            }
            Ok(PlannedCommand::tool_call(
                "issue_write",
                Value::Object(params),
            ))
        }
        IssuesSubcommand::CommentAdd(args) => Ok(PlannedCommand::tool_call(
            "issue_write",
            json!({
                "method": "add_comment",
                "owner": args.target.owner,
                "repo": args.target.repo,
                "index": args.target.index,
                "body": args.body
            }),
        )),
        IssuesSubcommand::CommentEdit(args) => Ok(PlannedCommand::tool_call(
            "issue_write",
            json!({
                "method": "edit_comment",
                "owner": args.target.owner,
                "repo": args.target.repo,
                "index": args.target.index,
                "commentID": args.comment_id,
                "body": args.body
            }),
        )),
        IssuesSubcommand::Labels(args) => Ok(PlannedCommand::tool_call(
            "issue_read",
            json!({
                "owner": args.owner,
                "repo": args.repo,
                "index": args.index,
                "method": "get_labels"
            }),
        )),
        IssuesSubcommand::LabelsAdd(args) => Ok(PlannedCommand::tool_call(
            "issue_write",
            json!({
                "method": "add_labels",
                "owner": args.target.owner,
                "repo": args.target.repo,
                "index": args.target.index,
                "labels": args.label_ids
            }),
        )),
        IssuesSubcommand::LabelRemove(args) => {
            require_yes(args.yes, "删除 issue label")?;
            Ok(PlannedCommand::tool_call(
                "issue_write",
                json!({
                    "method": "remove_label",
                    "owner": args.target.owner,
                    "repo": args.target.repo,
                    "index": args.target.index,
                    "label_id": args.label_id
                }),
            ))
        }
        IssuesSubcommand::LabelsReplace(args) => Ok(PlannedCommand::tool_call(
            "issue_write",
            json!({
                "method": "replace_labels",
                "owner": args.target.owner,
                "repo": args.target.repo,
                "index": args.target.index,
                "labels": args.label_ids
            }),
        )),
        IssuesSubcommand::LabelsClear(args) => {
            require_yes(args.yes, "清空 issue labels")?;
            Ok(PlannedCommand::tool_call(
                "issue_write",
                json!({
                    "method": "clear_labels",
                    "owner": args.target.owner,
                    "repo": args.target.repo,
                    "index": args.target.index
                }),
            ))
        }
        IssuesSubcommand::Time(command) => plan_issue_time(command),
    }
}

fn plan_issue_time(command: &IssueTimeCommand) -> Result<PlannedCommand> {
    match &command.command {
        IssueTimeSubcommand::List(args) => Ok(PlannedCommand::tool_call(
            "timetracking_read",
            json!({
                "method": "list_issue_times",
                "owner": args.target.owner,
                "repo": args.target.repo,
                "index": args.target.index,
                "page": args.page.page,
                "perPage": args.page.page_size
            }),
        )),
        IssueTimeSubcommand::Start(args) => Ok(PlannedCommand::tool_call(
            "timetracking_write",
            json!({
                "method": "start_stopwatch",
                "owner": args.owner,
                "repo": args.repo,
                "index": args.index
            }),
        )),
        IssueTimeSubcommand::Stop(args) => Ok(PlannedCommand::tool_call(
            "timetracking_write",
            json!({
                "method": "stop_stopwatch",
                "owner": args.owner,
                "repo": args.repo,
                "index": args.index
            }),
        )),
        IssueTimeSubcommand::ResetStopwatch(args) => {
            require_yes(args.yes, "清空 issue stopwatch")?;
            Ok(PlannedCommand::tool_call(
                "timetracking_write",
                json!({
                    "method": "delete_stopwatch",
                    "owner": args.target.owner,
                    "repo": args.target.repo,
                    "index": args.target.index
                }),
            ))
        }
        IssueTimeSubcommand::Add(args) => Ok(PlannedCommand::tool_call(
            "timetracking_write",
            json!({
                "method": "add_time",
                "owner": args.target.owner,
                "repo": args.target.repo,
                "index": args.target.index,
                "time": args.seconds
            }),
        )),
        IssueTimeSubcommand::Delete(args) => {
            require_yes(args.yes, "删除 issue time 记录")?;
            Ok(PlannedCommand::tool_call(
                "timetracking_write",
                json!({
                    "method": "delete_time",
                    "owner": args.target.owner,
                    "repo": args.target.repo,
                    "index": args.target.index,
                    "id": args.id
                }),
            ))
        }
    }
}

fn plan_labels(command: &LabelsCommand) -> Result<PlannedCommand> {
    match &command.command {
        LabelsSubcommand::RepoList(args) => Ok(PlannedCommand::tool_call(
            "label_read",
            json!({
                "method": "list_repo_labels",
                "owner": args.target.owner,
                "repo": args.target.repo,
                "page": args.page.page,
                "perPage": args.page.page_size
            }),
        )),
        LabelsSubcommand::RepoGet(args) => Ok(PlannedCommand::tool_call(
            "label_read",
            json!({
                "method": "get_repo_label",
                "owner": args.target.owner,
                "repo": args.target.repo,
                "id": args.id
            }),
        )),
        LabelsSubcommand::RepoCreate(args) => {
            let mut params = Map::new();
            params.insert("method".to_string(), json!("create_repo_label"));
            params.insert("owner".to_string(), json!(args.target.owner));
            params.insert("repo".to_string(), json!(args.target.repo));
            params.insert("name".to_string(), json!(args.name));
            params.insert("color".to_string(), json!(args.color));
            insert_optional_string(&mut params, "description", args.description.as_deref());
            if args.archived {
                params.insert("is_archived".to_string(), json!(true));
            }
            Ok(PlannedCommand::tool_call(
                "label_write",
                Value::Object(params),
            ))
        }
        LabelsSubcommand::RepoEdit(args) => {
            let mut params = Map::new();
            params.insert("method".to_string(), json!("edit_repo_label"));
            params.insert("owner".to_string(), json!(args.target.owner));
            params.insert("repo".to_string(), json!(args.target.repo));
            params.insert("id".to_string(), json!(args.id));
            insert_optional_string(&mut params, "name", args.name.as_deref());
            insert_optional_string(&mut params, "color", args.color.as_deref());
            insert_optional_string(&mut params, "description", args.description.as_deref());
            insert_optional_bool(&mut params, "is_archived", args.archived);
            Ok(PlannedCommand::tool_call(
                "label_write",
                Value::Object(params),
            ))
        }
        LabelsSubcommand::RepoDelete(args) => {
            require_yes(args.yes, "删除 repo label")?;
            Ok(PlannedCommand::tool_call(
                "label_write",
                json!({
                    "method": "delete_repo_label",
                    "owner": args.target.owner,
                    "repo": args.target.repo,
                    "id": args.id
                }),
            ))
        }
        LabelsSubcommand::OrgList(args) => Ok(PlannedCommand::tool_call(
            "label_read",
            json!({
                "method": "list_org_labels",
                "org": args.org,
                "page": args.page.page,
                "perPage": args.page.page_size
            }),
        )),
        LabelsSubcommand::OrgCreate(args) => {
            let mut params = Map::new();
            params.insert("method".to_string(), json!("create_org_label"));
            params.insert("org".to_string(), json!(args.org));
            params.insert("name".to_string(), json!(args.name));
            params.insert("color".to_string(), json!(args.color));
            insert_optional_string(&mut params, "description", args.description.as_deref());
            if args.exclusive {
                params.insert("exclusive".to_string(), json!(true));
            }
            Ok(PlannedCommand::tool_call(
                "label_write",
                Value::Object(params),
            ))
        }
        LabelsSubcommand::OrgEdit(args) => {
            let mut params = Map::new();
            params.insert("method".to_string(), json!("edit_org_label"));
            params.insert("org".to_string(), json!(args.org));
            params.insert("id".to_string(), json!(args.id));
            insert_optional_string(&mut params, "name", args.name.as_deref());
            insert_optional_string(&mut params, "color", args.color.as_deref());
            insert_optional_string(&mut params, "description", args.description.as_deref());
            insert_optional_bool(&mut params, "exclusive", args.exclusive);
            Ok(PlannedCommand::tool_call(
                "label_write",
                Value::Object(params),
            ))
        }
        LabelsSubcommand::OrgDelete(args) => {
            require_yes(args.yes, "删除 org label")?;
            Ok(PlannedCommand::tool_call(
                "label_write",
                json!({
                    "method": "delete_org_label",
                    "org": args.org,
                    "id": args.id
                }),
            ))
        }
    }
}

fn plan_milestones(command: &MilestonesCommand) -> Result<PlannedCommand> {
    match &command.command {
        MilestonesSubcommand::List(args) => {
            let mut params = Map::new();
            params.insert("method".to_string(), json!("list"));
            params.insert("owner".to_string(), json!(args.target.owner));
            params.insert("repo".to_string(), json!(args.target.repo));
            params.insert("page".to_string(), json!(args.page.page));
            params.insert("perPage".to_string(), json!(args.page.page_size));
            insert_optional_string(&mut params, "state", args.state.as_deref());
            insert_optional_string(&mut params, "name", args.name.as_deref());
            Ok(PlannedCommand::tool_call(
                "milestone_read",
                Value::Object(params),
            ))
        }
        MilestonesSubcommand::Get(args) => Ok(PlannedCommand::tool_call(
            "milestone_read",
            json!({
                "method": "get",
                "owner": args.target.owner,
                "repo": args.target.repo,
                "id": args.id
            }),
        )),
        MilestonesSubcommand::Create(args) => {
            let mut params = Map::new();
            params.insert("method".to_string(), json!("create"));
            params.insert("owner".to_string(), json!(args.target.owner));
            params.insert("repo".to_string(), json!(args.target.repo));
            params.insert("title".to_string(), json!(args.title));
            insert_optional_string(&mut params, "description", args.description.as_deref());
            insert_optional_string(&mut params, "due_on", args.due_on.as_deref());
            Ok(PlannedCommand::tool_call(
                "milestone_write",
                Value::Object(params),
            ))
        }
        MilestonesSubcommand::Edit(args) => {
            let mut params = Map::new();
            params.insert("method".to_string(), json!("edit"));
            params.insert("owner".to_string(), json!(args.target.owner));
            params.insert("repo".to_string(), json!(args.target.repo));
            params.insert("id".to_string(), json!(args.id));
            insert_optional_string(&mut params, "title", args.title.as_deref());
            insert_optional_string(&mut params, "description", args.description.as_deref());
            insert_optional_string(&mut params, "due_on", args.due_on.as_deref());
            insert_optional_string(&mut params, "state", args.state.as_deref());
            Ok(PlannedCommand::tool_call(
                "milestone_write",
                Value::Object(params),
            ))
        }
        MilestonesSubcommand::Delete(args) => {
            require_yes(args.yes, "删除 milestone")?;
            Ok(PlannedCommand::tool_call(
                "milestone_write",
                json!({
                    "method": "delete",
                    "owner": args.target.owner,
                    "repo": args.target.repo,
                    "id": args.id
                }),
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
        ReleasesSubcommand::Create(args) => {
            let mut params = Map::new();
            params.insert("owner".to_string(), json!(args.target.owner));
            params.insert("repo".to_string(), json!(args.target.repo));
            params.insert("tag_name".to_string(), json!(args.tag_name));
            params.insert("title".to_string(), json!(args.title));
            params.insert("target".to_string(), json!(args.target_ref));
            insert_optional_string(&mut params, "body", args.body.as_deref());
            if args.draft {
                params.insert("is_draft".to_string(), json!(true));
            }
            if args.pre_release {
                params.insert("is_pre_release".to_string(), json!(true));
            }
            Ok(PlannedCommand::tool_call(
                "create_release",
                Value::Object(params),
            ))
        }
        ReleasesSubcommand::Delete(args) => {
            require_yes(args.yes, "删除 release")?;
            Ok(PlannedCommand::tool_call(
                "delete_release",
                json!({
                    "owner": args.target.owner,
                    "repo": args.target.repo,
                    "id": args.id
                }),
            ))
        }
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
        TagsSubcommand::Create(args) => {
            let mut params = Map::new();
            params.insert("owner".to_string(), json!(args.target.owner));
            params.insert("repo".to_string(), json!(args.target.repo));
            params.insert("tag_name".to_string(), json!(args.tag_name));
            insert_optional_string(&mut params, "target", args.target_ref.as_deref());
            insert_optional_string(&mut params, "message", args.message.as_deref());
            Ok(PlannedCommand::tool_call(
                "create_tag",
                Value::Object(params),
            ))
        }
        TagsSubcommand::Delete(args) => {
            require_yes(args.yes, "删除 tag")?;
            Ok(PlannedCommand::tool_call(
                "delete_tag",
                json!({
                    "owner": args.target.owner,
                    "repo": args.target.repo,
                    "tag_name": args.tag_name
                }),
            ))
        }
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

fn insert_optional_u64(target: &mut Map<String, Value>, key: &str, value: Option<u64>) {
    if let Some(value) = value {
        target.insert(key.to_string(), json!(value));
    }
}

fn insert_optional_bool(target: &mut Map<String, Value>, key: &str, value: Option<bool>) {
    if let Some(value) = value {
        target.insert(key.to_string(), json!(value));
    }
}

fn insert_optional_string_list(target: &mut Map<String, Value>, key: &str, values: &[String]) {
    if !values.is_empty() {
        target.insert(key.to_string(), json!(values));
    }
}

fn insert_optional_u64_list(target: &mut Map<String, Value>, key: &str, values: &[u64]) {
    if !values.is_empty() {
        target.insert(key.to_string(), json!(values));
    }
}

fn require_yes(confirmed: bool, action: &str) -> Result<()> {
    if confirmed {
        Ok(())
    } else {
        bail!("危险操作需要显式传入 --yes 后才会执行: {action}");
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
