use anyhow::{Result, bail};
use clap::{Args, Subcommand};
use serde_json::{Value, json};

use crate::cli::PlannedCommand;

#[derive(Debug, Clone, Args)]
pub struct ProjectsCommand {
    #[command(subcommand)]
    pub command: ProjectCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum ProjectCommand {
    /// 列出项目；不指定 scope 时读取当前用户项目
    List(ProjectListArgs),
    /// 读取项目
    Get(ProjectTarget),
    /// 列出项目列
    Columns(ProjectTarget),
    /// 列出列内 issue
    ColumnIssues(ColumnIssuesArgs),
    /// 创建项目
    Create(ProjectCreateArgs),
    /// 修改项目
    Update(ProjectUpdateArgs),
    /// 删除项目
    Delete(ProjectDeleteArgs),
    /// 创建列
    ColumnCreate(ColumnCreateArgs),
    /// 更新列
    ColumnUpdate(ColumnUpdateArgs),
    /// 删除列
    ColumnDelete(ColumnDeleteArgs),
    /// 加入 issue，或将已有 issue 移动到指定列
    IssueAdd(ProjectIssueArgs),
    /// 从项目移除 issue
    IssueRemove(ProjectIssueRemoveArgs),
}

#[derive(Debug, Clone, Args)]
pub struct ProjectScope {
    /// 仓库 owner（与 --repo 成对使用）
    #[arg(long, requires = "repo", conflicts_with = "org")]
    pub owner: Option<String>,
    #[arg(long, requires = "owner", conflicts_with = "org")]
    pub repo: Option<String>,
    /// 组织项目；不传 scope 时操作当前用户项目
    #[arg(long)]
    pub org: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct ProjectTarget {
    #[command(flatten)]
    pub scope: ProjectScope,
    #[arg(long)]
    pub project_id: u64,
}

#[derive(Debug, Clone, Args)]
pub struct ProjectListArgs {
    #[command(flatten)]
    pub scope: ProjectScope,
    #[arg(long, default_value = "all", value_parser = ["open", "closed", "all"])]
    pub state: String,
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u32).range(1..))]
    pub page: u32,
    #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u32).range(1..))]
    pub page_size: u32,
}

#[derive(Debug, Clone, Args)]
pub struct ColumnTarget {
    #[command(flatten)]
    pub target: ProjectTarget,
    #[arg(long)]
    pub column_id: u64,
}

#[derive(Debug, Clone, Args)]
pub struct ColumnIssuesArgs {
    #[command(flatten)]
    pub target: ColumnTarget,
    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u32).range(1..))]
    pub page: u32,
    #[arg(long, default_value_t = 30, value_parser = clap::value_parser!(u32).range(1..))]
    pub page_size: u32,
}

#[derive(Debug, Clone, Args)]
pub struct ProjectCreateArgs {
    #[command(flatten)]
    pub scope: ProjectScope,
    #[arg(long)]
    pub title: String,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long, value_parser = ["none", "basic_kanban", "bug_triage"])]
    pub template_type: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct ProjectUpdateArgs {
    #[command(flatten)]
    pub target: ProjectTarget,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long, value_parser = ["open", "closed"])]
    pub state: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct ProjectDeleteArgs {
    #[command(flatten)]
    pub target: ProjectTarget,
    #[arg(long)]
    pub yes: bool,
}

#[derive(Debug, Clone, Args)]
pub struct ColumnCreateArgs {
    #[command(flatten)]
    pub target: ProjectTarget,
    #[arg(long)]
    pub title: String,
}

#[derive(Debug, Clone, Args)]
pub struct ColumnUpdateArgs {
    #[command(flatten)]
    pub target: ColumnTarget,
    #[arg(long)]
    pub title: String,
}

#[derive(Debug, Clone, Args)]
pub struct ColumnDeleteArgs {
    #[command(flatten)]
    pub target: ColumnTarget,
    #[arg(long)]
    pub yes: bool,
}

#[derive(Debug, Clone, Args)]
pub struct ProjectIssueArgs {
    #[command(flatten)]
    pub target: ProjectTarget,
    /// issue 全局 ID，不是仓库内编号
    #[arg(long)]
    pub issue_id: u64,
    #[arg(long)]
    pub column_id: Option<u64>,
}

#[derive(Debug, Clone, Args)]
pub struct ProjectIssueRemoveArgs {
    #[command(flatten)]
    pub target: ProjectTarget,
    /// issue 全局 ID，不是仓库内编号
    #[arg(long)]
    pub issue_id: u64,
    #[arg(long)]
    pub yes: bool,
}

fn scope_params(scope: &ProjectScope, method: &str) -> Value {
    let mut params = json!({"method":method});
    for (name, value) in [
        ("owner", &scope.owner),
        ("repo", &scope.repo),
        ("org", &scope.org),
    ] {
        if let Some(value) = value {
            params[name] = json!(value);
        }
    }
    params
}

fn target_params(target: &ProjectTarget, method: &str) -> Value {
    let mut params = scope_params(&target.scope, method);
    params["project_id"] = json!(target.project_id);
    params
}

fn column_params(target: &ColumnTarget, method: &str) -> Value {
    let mut params = target_params(&target.target, method);
    params["column_id"] = json!(target.column_id);
    params
}

fn confirm(yes: bool) -> Result<()> {
    if !yes {
        bail!("删除或移除项目内容需要显式传入 --yes");
    }
    Ok(())
}

pub fn plan(command: &ProjectsCommand) -> Result<PlannedCommand> {
    use ProjectCommand::*;
    let (tool, params) = match &command.command {
        List(a) => {
            let mut p = scope_params(&a.scope, "list");
            p["state"] = json!(a.state);
            p["page"] = json!(a.page);
            p["per_page"] = json!(a.page_size);
            ("project_read", p)
        }
        Get(a) => ("project_read", target_params(a, "get")),
        Columns(a) => ("project_read", target_params(a, "list_columns")),
        ColumnIssues(a) => {
            let mut p = column_params(&a.target, "list_column_issues");
            p["page"] = json!(a.page);
            p["per_page"] = json!(a.page_size);
            ("project_read", p)
        }
        Create(a) => {
            let mut p = scope_params(&a.scope, "create");
            p["title"] = json!(a.title);
            if let Some(v) = &a.description {
                p["description"] = json!(v);
            }
            if let Some(v) = &a.template_type {
                p["template_type"] = json!(v);
            }
            ("project_write", p)
        }
        Update(a) => {
            if a.title.is_none() && a.description.is_none() && a.state.is_none() {
                bail!("需要指定 --title、--description 或 --state");
            }
            let mut p = target_params(&a.target, "update");
            for (key, v) in [
                ("title", &a.title),
                ("description", &a.description),
                ("state", &a.state),
            ] {
                if let Some(v) = v {
                    p[key] = json!(v);
                }
            }
            ("project_write", p)
        }
        Delete(a) => {
            confirm(a.yes)?;
            ("project_write", target_params(&a.target, "delete"))
        }
        ColumnCreate(a) => {
            let mut p = target_params(&a.target, "create_column");
            p["title"] = json!(a.title);
            ("project_write", p)
        }
        ColumnUpdate(a) => {
            let mut p = column_params(&a.target, "update_column");
            p["title"] = json!(a.title);
            ("project_write", p)
        }
        ColumnDelete(a) => {
            confirm(a.yes)?;
            ("project_write", column_params(&a.target, "delete_column"))
        }
        IssueAdd(a) => {
            let mut p = target_params(&a.target, "add_issue");
            p["issue_id"] = json!(a.issue_id);
            if let Some(v) = a.column_id {
                p["column_id"] = json!(v);
            }
            ("project_write", p)
        }
        IssueRemove(a) => {
            confirm(a.yes)?;
            let mut p = target_params(&a.target, "remove_issue");
            p["issue_id"] = json!(a.issue_id);
            ("project_write", p)
        }
    };
    Ok(PlannedCommand::tool_call(tool, params))
}
