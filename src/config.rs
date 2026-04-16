use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GiteaServerConfig {
    pub command: String,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct CodexConfig {
    #[serde(default)]
    mcp_servers: BTreeMap<String, McpServer>,
}

#[derive(Debug, Deserialize)]
struct McpServer {
    command: Option<String>,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    env: BTreeMap<String, String>,
}

pub fn default_codex_config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("无法定位当前用户主目录")?;
    Ok(home.join(".codex").join("config.toml"))
}

pub fn load_gitea_server_config(path: &Path) -> Result<GiteaServerConfig> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("读取 Codex 配置失败: {}", path.display()))?;
    let config: CodexConfig =
        toml::from_str(&raw).with_context(|| format!("解析 Codex 配置失败: {}", path.display()))?;
    let server = config
        .mcp_servers
        .get("gitea")
        .context("未找到 [mcp_servers.gitea] 配置")?;

    let command = server
        .command
        .as_deref()
        .context("当前 gitea MCP 配置缺少 command，暂仅支持 stdio 类型 server")?;

    if command.trim().is_empty() {
        bail!("gitea MCP server command 不能为空");
    }

    Ok(GiteaServerConfig {
        command: command.to_string(),
        args: server.args.clone(),
        env: server.env.clone(),
    })
}
