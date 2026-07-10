use std::{env, ffi::OsString, fs};

use anyhow::{Result, anyhow};
use clap::error::ErrorKind;
use serde_json::{Value, json};
use which::which;

use crate::{
    cli::{Cli, PlannedCommand, ResultFilter, plan_command},
    config::{
        default_config_path, load_config_view, load_gitea_server_config, render_init_config,
        resolve_config_path,
    },
    mcp::McpSession,
    output::{filter_comments_by_ids, select_fields},
};

pub fn run<I, T>(args: I) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(error) => {
            let exit_code = match error.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => 0,
                _ => 2,
            };
            let _ = error.print();
            return exit_code;
        }
    };

    match run_cli(&cli) {
        Ok(result) => {
            let output = match shape_output(&result, &cli) {
                Ok(output) => output,
                Err(error) => {
                    if cli.json {
                        print_output(
                            &json!({
                                "ok": false,
                                "error": error.to_string()
                            }),
                            true,
                        );
                    } else {
                        eprintln!("{error:#}");
                    }
                    return 1;
                }
            };
            print_output(&output, cli.json);
            0
        }
        Err(error) => {
            if cli.json {
                print_output(
                    &json!({
                        "ok": false,
                        "error": error.to_string()
                    }),
                    true,
                );
            } else {
                eprintln!("{error:#}");
            }
            1
        }
    }
}

fn shape_output(value: &Value, cli: &Cli) -> Result<Value> {
    if !cli.json || cli.fields.is_empty() {
        return Ok(value.clone());
    }

    select_fields(value, &cli.fields)
}

fn run_cli(cli: &Cli) -> Result<Value> {
    match plan_command(cli)? {
        PlannedCommand::Doctor => run_doctor(cli),
        PlannedCommand::ConfigPath => run_config_path(cli),
        PlannedCommand::ConfigInit { host, force } => run_config_init(cli, &host, force),
        PlannedCommand::ConfigShow => run_config_show(cli),
        PlannedCommand::ConfigValidate => run_config_validate(cli),
        PlannedCommand::ToolsList => {
            let mut session = start_session(cli)?;
            let result = session.list_tools()?;
            Ok(json!({
                "ok": true,
                "kind": "tools_list",
                "result": result
            }))
        }
        PlannedCommand::ToolCall {
            tool,
            params,
            filter,
        } => {
            let mut session = start_session(cli)?;
            let result = session.call_tool(&tool, params)?;
            let wrapped = json!({
                "ok": true,
                "kind": "tool_call",
                "tool": tool,
                "result": result
            });
            apply_result_filter(&wrapped, filter.as_ref())
        }
        PlannedCommand::Resolve { result } => Ok(json!({
            "ok": true,
            "kind": "resolve",
            "result": result
        })),
    }
}

fn apply_result_filter(value: &Value, filter: Option<&ResultFilter>) -> Result<Value> {
    let Some(filter) = filter else {
        return Ok(value.clone());
    };

    let mut next = value.clone();
    let parsed = next
        .get_mut("result")
        .and_then(Value::as_object_mut)
        .and_then(|result| result.get_mut("parsed"))
        .ok_or_else(|| anyhow!("过滤评论失败: 找不到 result.parsed"))?;

    *parsed = filter_comments_by_ids(parsed, &filter.comment_ids)?;
    Ok(next)
}

fn start_session(cli: &Cli) -> Result<McpSession> {
    let config_path = resolve_runtime_config_path(cli)?;
    let config = load_gitea_server_config(&config_path, cli.profile.as_deref())?;
    McpSession::start(&config)
}

fn run_config_path(cli: &Cli) -> Result<Value> {
    let path = resolve_runtime_config_path(cli)?;
    Ok(json!({
        "ok": true,
        "kind": "config_path",
        "config": {
            "path": path,
            "exists": path.exists(),
            "source": config_source(cli),
            "profile": cli.profile
        }
    }))
}

fn run_config_init(cli: &Cli, host: &str, force: bool) -> Result<Value> {
    let profile = cli
        .profile
        .as_deref()
        .ok_or_else(|| anyhow!("config init 需要 --profile <name>"))?;
    let path = resolve_runtime_config_path(cli)?;

    if path.exists() && !force {
        return Err(anyhow!(
            "配置文件已存在: {}；如需覆盖请添加 --force",
            path.display()
        ));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| anyhow!("创建配置目录失败: {}: {error}", parent.display()))?;
    }

    let content = render_init_config(profile, host)?;
    fs::write(&path, content)
        .map_err(|error| anyhow!("写入配置文件失败: {}: {error}", path.display()))?;

    Ok(json!({
        "ok": true,
        "kind": "config_init",
        "config": {
            "path": path,
            "source": config_source(cli),
            "profile": profile,
            "host": host
        }
    }))
}

fn run_config_show(cli: &Cli) -> Result<Value> {
    let path = resolve_runtime_config_path(cli)?;
    let view = load_config_view(&path, cli.profile.as_deref())?;
    let mut config = json!(view);
    if let Some(object) = config.as_object_mut() {
        object.insert("source".to_string(), json!(config_source(cli)));
    }

    Ok(json!({
        "ok": true,
        "kind": "config_show",
        "config": config
    }))
}

fn run_config_validate(cli: &Cli) -> Result<Value> {
    let path = resolve_runtime_config_path(cli)?;
    let config = load_gitea_server_config(&path, cli.profile.as_deref())?;
    let command_exists = which(&config.command).is_ok();

    if !command_exists {
        return Err(anyhow!("MCP server 命令不在 PATH 中: {}", config.command));
    }

    Ok(json!({
        "ok": true,
        "kind": "config_validate",
        "config": {
            "path": config.metadata.path,
            "source": config_source(cli),
            "profile": config.metadata.profile,
            "host": config.metadata.host,
            "auth": config.metadata.auth,
            "warnings": config.metadata.warnings
        },
        "server": {
            "command": config.command,
            "exists_on_path": command_exists,
            "args": redact_args(&config.args),
            "env_keys": sorted_env_keys(&config.env)
        }
    }))
}

fn run_doctor(cli: &Cli) -> Result<Value> {
    let config_path = resolve_runtime_config_path(cli)?;
    let config_exists = config_path.exists();
    let mut issues = Vec::new();
    let mut warnings = Vec::new();

    let mut profile_name = None::<String>;
    let mut host = None::<String>;
    let mut auth = None::<Value>;
    let mut command_name = None::<String>;
    let mut command_exists = false;
    let mut redacted_args: Vec<String> = Vec::new();
    let mut env_keys: Vec<String> = Vec::new();
    let mut startup_ok = false;
    let mut auth_ok = false;
    let mut tools_count = None::<usize>;

    if !config_exists {
        issues.push(format!("gitea-cli 配置不存在: {}", config_path.display()));
    } else {
        match load_gitea_server_config(&config_path, cli.profile.as_deref()) {
            Ok(config) => {
                profile_name = Some(config.metadata.profile.clone());
                host = Some(config.metadata.host.clone());
                auth = Some(json!(config.metadata.auth));
                warnings.extend(config.metadata.warnings.clone());
                command_name = Some(config.command.clone());
                command_exists = which(&config.command).is_ok();
                redacted_args = redact_args(&config.args);
                env_keys = sorted_env_keys(&config.env);

                if !command_exists {
                    issues.push(format!("MCP server 命令不在 PATH 中: {}", config.command));
                } else {
                    match McpSession::start(&config) {
                        Ok(mut session) => {
                            match session.list_tools() {
                                Ok(result) => {
                                    startup_ok = true;
                                    tools_count = result
                                        .get("tools")
                                        .and_then(Value::as_array)
                                        .map(|items| items.len());
                                }
                                Err(error) => {
                                    issues.push(format!("MCP tools/list 探测失败: {error}"));
                                }
                            }

                            match session.call_tool("get_me", json!({})) {
                                Ok(_) => {
                                    auth_ok = true;
                                }
                                Err(error) => {
                                    issues.push(format!("MCP 认证探测失败: {error}"));
                                }
                            }
                        }
                        Err(error) => {
                            issues.push(format!("MCP 启动探测失败: {error}"));
                        }
                    }
                }
            }
            Err(error) => issues.push(error.to_string()),
        }
    }

    Ok(json!({
        "ok": issues.is_empty(),
        "kind": "doctor",
        "cli": {
            "name": env!("CARGO_PKG_NAME"),
            "version": env!("CARGO_PKG_VERSION")
        },
        "config": {
            "path": config_path,
            "exists": config_exists,
            "source": config_source(cli),
            "profile": profile_name,
            "host": host,
            "auth": auth
        },
        "server": {
            "command": command_name,
            "exists_on_path": command_exists,
            "args": redacted_args,
            "env_keys": env_keys
        },
        "startup": {
            "ok": startup_ok
        },
        "auth": {
            "ok": auth_ok
        },
        "tools_count": tools_count,
        "warnings": warnings,
        "issues": issues
    }))
}

fn resolve_runtime_config_path(cli: &Cli) -> Result<std::path::PathBuf> {
    if let Some(path) = &cli.config {
        return resolve_config_path(Some(path));
    }

    resolve_config_path(None).or_else(|_| default_config_path())
}

fn sorted_env_keys(env: &std::collections::BTreeMap<String, String>) -> Vec<String> {
    let mut keys: Vec<String> = env.keys().cloned().collect();
    keys.sort();
    keys
}

fn config_source(cli: &Cli) -> &'static str {
    if cli.config.is_some() {
        "flag"
    } else if env::var("GITEA_CLI_CONFIG")
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
    {
        "env"
    } else {
        "default"
    }
}

fn redact_args(args: &[String]) -> Vec<String> {
    let mut redacted = Vec::with_capacity(args.len());
    let mut redact_next = false;

    for arg in args {
        if redact_next {
            redacted.push("***REDACTED***".to_string());
            redact_next = false;
            continue;
        }

        let lower = arg.to_ascii_lowercase();
        if lower == "--token"
            || lower == "--password"
            || lower == "--secret"
            || lower == "--access-token"
            || lower.ends_with("token")
        {
            redacted.push(arg.clone());
            redact_next = true;
            continue;
        }

        if let Some((key, _)) = arg.split_once('=') {
            let lower_key = key.to_ascii_lowercase();
            if lower_key.contains("token")
                || lower_key.contains("secret")
                || lower_key.contains("password")
                || lower_key.contains("key")
            {
                redacted.push(format!("{key}=***REDACTED***"));
                continue;
            }
        }

        redacted.push(arg.clone());
    }

    redacted
}

fn print_output(value: &Value, compact: bool) {
    let rendered = if compact {
        serde_json::to_string(value)
    } else {
        serde_json::to_string_pretty(value)
    }
    .unwrap_or_else(|_| "{\"ok\":false,\"error\":\"render json failed\"}".to_string());

    println!("{rendered}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Command, ResultFilter};

    #[test]
    fn apply_result_filter_keeps_matching_comments_before_fields() {
        let value = serde_json::json!({
            "ok": true,
            "kind": "tool_call",
            "result": {
                "parsed": [
                    {"id": 88, "body": "first", "user": {"login": "fanbuz"}},
                    {"id": 99, "body": "second", "user": {"login": "agent"}}
                ]
            }
        });

        let filtered =
            apply_result_filter(&value, Some(&ResultFilter::comment_ids(vec![99]))).unwrap();
        let shaped = shape_output(
            &filtered,
            &Cli {
                config: None,
                profile: None,
                json: true,
                fields: vec![
                    "result.parsed.0.id".to_string(),
                    "result.parsed.0.user.login".to_string(),
                ],
                command: Command::Doctor,
            },
        )
        .unwrap();

        assert_eq!(
            shaped,
            serde_json::json!({
                "result": {
                    "parsed": [
                        {"id": 99, "user": {"login": "agent"}}
                    ]
                }
            })
        );
    }

    #[test]
    fn apply_result_filter_returns_empty_array_when_no_comment_matches() {
        let value = serde_json::json!({
            "ok": true,
            "kind": "tool_call",
            "result": {
                "parsed": [
                    {"id": 88, "body": "first"}
                ]
            }
        });

        let filtered =
            apply_result_filter(&value, Some(&ResultFilter::comment_ids(vec![999]))).unwrap();

        assert_eq!(filtered["result"]["parsed"], serde_json::json!([]));
    }
}
