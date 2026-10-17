use std::{collections::HashSet, fs};

use anyhow::{Context, Result, bail};
use clap::Args;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::cli::{PlannedCommand, RepoTargetArgs};

#[derive(Debug, Clone, Args)]
pub struct FilesWriteArgs {
    #[command(flatten)]
    pub target: RepoTargetArgs,
    #[arg(long)]
    pub branch: String,
    #[arg(long)]
    pub message: String,
    #[arg(long)]
    pub new_branch: Option<String>,
    /// 文件操作数组：JSON 或 @本地文件；内容为原始文本
    #[arg(long)]
    pub files: String,
    /// 包含删除操作时必须确认
    #[arg(long)]
    pub yes: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Edit {
    old_string: String,
    new_string: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    replace_all: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FileChange {
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    edits: Option<Vec<Edit>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sha: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    from_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delete: Option<bool>,
}

fn parse<T: serde::de::DeserializeOwned>(raw: &str) -> Result<T> {
    let source = if let Some(path) = raw.strip_prefix('@') {
        fs::read_to_string(path).with_context(|| format!("读取 JSON 文件失败: {path}"))?
    } else {
        raw.to_owned()
    };
    serde_json::from_str(&source).context("文件操作 JSON 格式无效")
}

fn validate_edits(edits: &[Edit]) -> Result<()> {
    if edits.is_empty() || edits.iter().any(|edit| edit.old_string.is_empty()) {
        bail!("edits 必须是非空数组，old_string 不能为空");
    }
    Ok(())
}

pub fn parse_edits(raw: &str) -> Result<Value> {
    let edits: Vec<Edit> = parse(raw)?;
    validate_edits(&edits)?;
    Ok(serde_json::to_value(edits)?)
}

pub fn plan(args: &FilesWriteArgs) -> Result<PlannedCommand> {
    let files: Vec<FileChange> = parse(&args.files)?;
    if files.is_empty() {
        bail!("--files 必须包含至少一个文件操作");
    }
    let mut touched = HashSet::new();
    for file in &files {
        for path in std::iter::once(&file.path).chain(file.from_path.iter()) {
            if path.trim().is_empty() || !touched.insert(path) {
                bail!("文件路径不能为空或重复: {path}");
            }
        }
        if let Some(edits) = &file.edits {
            validate_edits(edits)?;
        }
        let has_edits = file.edits.is_some();
        if file.content.is_some() && has_edits {
            bail!("content 与 edits 不能同时传入");
        }
        if file.delete == Some(true) {
            if !args.yes {
                bail!("批量删除文件需要显式传入 --yes");
            }
            if file.content.is_some() || has_edits || file.from_path.is_some() {
                bail!("delete 不能与 content、edits 或 from_path 同时传入");
            }
        } else if file.content.is_none() && !has_edits && file.from_path.is_none() {
            bail!("文件操作需要 content、edits、from_path 或 delete");
        }
        // Edits fetch the current source SHA upstream; rename/delete must provide one.
        if (file.delete == Some(true) || file.from_path.is_some())
            && !has_edits
            && file.sha.as_deref().is_none_or(|s| s.trim().is_empty())
        {
            bail!("重命名或删除文件需要 sha");
        }
    }
    let mut params = json!({"owner":args.target.owner,"repo":args.target.repo,
        "branch_name":args.branch,"message":args.message,"files":files});
    if let Some(branch) = &args.new_branch {
        params["new_branch_name"] = json!(branch);
    }
    Ok(PlannedCommand::tool_call("create_or_update_file", params))
}
