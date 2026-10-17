use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

const CONFIG_ENV_VAR: &str = "GITEA_CLI_CONFIG";
const DEFAULT_TOKEN_ENV: &str = "GITEA_ACCESS_TOKEN";
const DEFAULT_TOKEN_FILE_ENV: &str = "GITEA_ACCESS_TOKEN_FILE";
const DEFAULT_HOST_ENV: &str = "GITEA_HOST";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GiteaServerConfig {
    pub command: String,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub metadata: ConfigMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConfigMetadata {
    pub path: PathBuf,
    pub profile: String,
    pub host: String,
    pub auth: AuthSummary,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthSummary {
    #[serde(rename = "type")]
    pub kind: String,
    pub token_env: Option<String>,
    pub token_file: Option<PathBuf>,
    pub inline_token: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConfigView {
    pub path: PathBuf,
    pub exists: bool,
    pub active_profile: Option<String>,
    pub profile: String,
    pub host: String,
    pub server: ServerView,
    pub auth: AuthSummary,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServerView {
    pub command: String,
    pub args: Vec<String>,
    pub env_keys: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GiteaCliConfig {
    version: Option<u32>,
    active_profile: Option<String>,
    #[serde(default)]
    profiles: BTreeMap<String, ProfileConfig>,
}

#[derive(Debug, Deserialize)]
struct ProfileConfig {
    host: Option<String>,
    mcp: Option<McpConfig>,
    auth: Option<AuthConfig>,
}

#[derive(Debug, Deserialize)]
struct McpConfig {
    command: Option<String>,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    env: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct AuthConfig {
    #[serde(rename = "type")]
    kind: Option<String>,
    token_env: Option<String>,
    token_file: Option<String>,
    token: Option<String>,
}

pub fn default_config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("无法定位当前用户主目录")?;
    Ok(home.join(".config").join("gitea-cli").join("config.toml"))
}

pub fn resolve_config_path(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return expand_path(path);
    }

    if let Ok(raw) = env::var(CONFIG_ENV_VAR)
        && !raw.trim().is_empty()
    {
        return expand_path(Path::new(raw.trim()));
    }

    default_config_path()
}

pub fn load_gitea_server_config(
    path: &Path,
    profile_override: Option<&str>,
) -> Result<GiteaServerConfig> {
    let config = read_config(path)?;
    let (profile_name, profile) = select_profile(&config, profile_override)?;
    let host = required_trimmed(
        profile.host.as_deref(),
        format!("profile `{profile_name}` 缺少 host"),
    )?;
    let mcp = profile
        .mcp
        .as_ref()
        .with_context(|| format!("profile `{profile_name}` 缺少 [profiles.{profile_name}.mcp]"))?;
    let command = required_trimmed(
        mcp.command.as_deref(),
        format!("profile `{profile_name}` 缺少 mcp.command"),
    )?;
    let auth = profile
        .auth
        .as_ref()
        .with_context(|| format!("profile `{profile_name}` 缺少 [profiles.{profile_name}.auth]"))?;

    let mut env_vars = mcp.env.clone();
    env_vars.insert(DEFAULT_HOST_ENV.to_string(), host.clone());
    let (auth_summary, warnings) = apply_auth_env(auth, &mut env_vars)?;

    Ok(GiteaServerConfig {
        command,
        args: mcp.args.clone(),
        env: env_vars,
        metadata: ConfigMetadata {
            path: path.to_path_buf(),
            profile: profile_name,
            host,
            auth: auth_summary,
            warnings,
        },
    })
}

pub fn load_config_view(path: &Path, profile_override: Option<&str>) -> Result<ConfigView> {
    let config = read_config(path)?;
    let active_profile = config.active_profile.clone();
    let (profile_name, profile) = select_profile(&config, profile_override)?;
    let host = required_trimmed(
        profile.host.as_deref(),
        format!("profile `{profile_name}` 缺少 host"),
    )?;
    let mcp = profile
        .mcp
        .as_ref()
        .with_context(|| format!("profile `{profile_name}` 缺少 [profiles.{profile_name}.mcp]"))?;
    let command = required_trimmed(
        mcp.command.as_deref(),
        format!("profile `{profile_name}` 缺少 mcp.command"),
    )?;
    let auth = profile
        .auth
        .as_ref()
        .with_context(|| format!("profile `{profile_name}` 缺少 [profiles.{profile_name}.auth]"))?;
    let (auth_summary, warnings) = summarize_auth(auth)?;
    let mut env_keys: Vec<String> = mcp.env.keys().cloned().collect();
    env_keys.sort();

    Ok(ConfigView {
        path: path.to_path_buf(),
        exists: path.exists(),
        active_profile,
        profile: profile_name,
        host,
        server: ServerView {
            command,
            args: mcp.args.clone(),
            env_keys,
        },
        auth: auth_summary,
        warnings,
    })
}

pub fn render_init_config(profile: &str, host: &str) -> Result<String> {
    let profile = required_trimmed(Some(profile), "profile 不能为空")?;
    let host = required_trimmed(Some(host), "host 不能为空")?;

    Ok(format!(
        r#"version = 1
active_profile = "{profile}"

[profiles.{profile}]
host = "{host}"

[profiles.{profile}.mcp]
command = "gitea-mcp-server"
args = ["-t", "stdio"]

[profiles.{profile}.mcp.env]

[profiles.{profile}.auth]
type = "token_env"
token_env = "{DEFAULT_TOKEN_ENV}"
"#
    ))
}

fn read_config(path: &Path) -> Result<GiteaCliConfig> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("读取 gitea-cli 配置失败: {}", path.display()))?;
    let config: GiteaCliConfig = toml::from_str(&raw)
        .with_context(|| format!("解析 gitea-cli 配置失败: {}", path.display()))?;

    match config.version {
        Some(1) => {}
        Some(version) => bail!("不支持的 gitea-cli 配置 version: {version}"),
        None => bail!("gitea-cli 配置缺少 version = 1"),
    }

    if config.profiles.is_empty() {
        bail!("gitea-cli 配置缺少 [profiles.<name>] 配置");
    }

    Ok(config)
}

fn select_profile<'a>(
    config: &'a GiteaCliConfig,
    profile_override: Option<&str>,
) -> Result<(String, &'a ProfileConfig)> {
    let profile_name = if let Some(profile) = profile_override {
        required_trimmed(Some(profile), "--profile 不能为空")?
    } else if let Some(profile) = config.active_profile.as_deref() {
        required_trimmed(Some(profile), "active_profile 不能为空")?
    } else if config.profiles.len() == 1 {
        config
            .profiles
            .keys()
            .next()
            .expect("profiles is not empty")
            .clone()
    } else {
        bail!("gitea-cli 配置缺少 active_profile");
    };

    let profile = config
        .profiles
        .get(&profile_name)
        .with_context(|| format!("未找到 profile `{profile_name}`"))?;

    Ok((profile_name, profile))
}

fn apply_auth_env(
    auth: &AuthConfig,
    target: &mut BTreeMap<String, String>,
) -> Result<(AuthSummary, Vec<String>)> {
    let (summary, warnings) = summarize_auth(auth)?;

    match summary.kind.as_str() {
        "token_env" => {
            let key = summary
                .token_env
                .as_deref()
                .context("auth.type=token_env 时必须配置 token_env")?;
            let token =
                env::var(key).with_context(|| format!("认证环境变量未设置或不可读取: {key}"))?;
            if token.trim().is_empty() {
                bail!("认证环境变量为空: {key}");
            }
            target.insert(DEFAULT_TOKEN_ENV.to_string(), token);
        }
        "token_file" => {
            let token_file = summary
                .token_file
                .as_ref()
                .context("auth.type=token_file 时必须配置 token_file")?;
            let token = fs::read_to_string(token_file)
                .with_context(|| format!("读取 token_file 失败: {}", token_file.display()))?;
            if token.trim().is_empty() {
                bail!("token_file 内容为空: {}", token_file.display());
            }
            target.insert(
                DEFAULT_TOKEN_FILE_ENV.to_string(),
                token_file.display().to_string(),
            );
        }
        "token" => {
            let token = auth
                .token
                .as_deref()
                .context("auth.type=token 时必须配置 token")?;
            if token.trim().is_empty() {
                bail!("auth.token 不能为空");
            }
            target.insert(DEFAULT_TOKEN_ENV.to_string(), token.to_string());
        }
        other => bail!("不支持的 auth.type: {other}"),
    }

    Ok((summary, warnings))
}

fn summarize_auth(auth: &AuthConfig) -> Result<(AuthSummary, Vec<String>)> {
    let kind = required_trimmed(auth.kind.as_deref(), "auth.type 不能为空")?;
    let mut warnings = Vec::new();

    let summary = match kind.as_str() {
        "token_env" => {
            let token_env = required_trimmed(
                auth.token_env.as_deref(),
                "auth.type=token_env 时必须配置 token_env",
            )?;
            AuthSummary {
                kind,
                token_env: Some(token_env),
                token_file: None,
                inline_token: false,
            }
        }
        "token_file" => {
            let token_file = required_trimmed(
                auth.token_file.as_deref(),
                "auth.type=token_file 时必须配置 token_file",
            )?;
            AuthSummary {
                kind,
                token_env: None,
                token_file: Some(expand_path(Path::new(&token_file))?),
                inline_token: false,
            }
        }
        "token" => {
            warnings
                .push("auth.token 使用明文 token，建议改用 token_env 或 token_file".to_string());
            AuthSummary {
                kind,
                token_env: None,
                token_file: None,
                inline_token: true,
            }
        }
        other => bail!("不支持的 auth.type: {other}"),
    };

    Ok((summary, warnings))
}

fn required_trimmed(value: Option<&str>, message: impl Into<String>) -> Result<String> {
    let value = value.map(str::trim).filter(|value| !value.is_empty());
    match value {
        Some(value) => Ok(value.to_string()),
        None => bail!(message.into()),
    }
}

fn expand_path(path: &Path) -> Result<PathBuf> {
    let raw = path.to_string_lossy();
    if raw == "~" {
        let home = dirs::home_dir().context("无法定位当前用户主目录")?;
        return Ok(home);
    }

    if let Some(rest) = raw.strip_prefix("~/") {
        let home = dirs::home_dir().context("无法定位当前用户主目录")?;
        return Ok(home.join(rest));
    }

    Ok(path.to_path_buf())
}
