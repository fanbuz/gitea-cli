use std::ffi::OsString;

use anyhow::{Result, anyhow};
use clap::error::ErrorKind;
use serde_json::{Value, json};
use which::which;

use crate::{
    cli::{Cli, PlannedCommand, ResultFilter, plan_command},
    config::{default_codex_config_path, load_gitea_server_config},
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
        PlannedCommand::Doctor => run_doctor(),
        PlannedCommand::ToolsList => {
            let mut session = start_session()?;
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
            let mut session = start_session()?;
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

fn start_session() -> Result<McpSession> {
    let config_path = default_codex_config_path()?;
    let config = load_gitea_server_config(&config_path)?;
    McpSession::start(&config)
}

fn run_doctor() -> Result<Value> {
    let config_path = default_codex_config_path()?;
    let config_exists = config_path.exists();
    let mut issues = Vec::new();

    let mut command_name = None::<String>;
    let mut command_exists = false;
    let mut redacted_args: Vec<String> = Vec::new();
    let mut env_keys: Vec<String> = Vec::new();
    let mut startup_ok = false;
    let mut tools_count = None::<usize>;

    if !config_exists {
        issues.push(format!("Codex 配置不存在: {}", config_path.display()));
    } else {
        match load_gitea_server_config(&config_path) {
            Ok(config) => {
                command_name = Some(config.command.clone());
                command_exists = which(&config.command).is_ok();
                redacted_args = redact_args(&config.args);
                env_keys = config.env.keys().cloned().collect();
                env_keys.sort();

                if !command_exists {
                    issues.push(format!("MCP server 命令不在 PATH 中: {}", config.command));
                } else {
                    match McpSession::start(&config).and_then(|mut session| session.list_tools()) {
                        Ok(result) => {
                            startup_ok = true;
                            tools_count = result
                                .get("tools")
                                .and_then(Value::as_array)
                                .map(|items| items.len());
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
            "exists": config_exists
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
        "tools_count": tools_count,
        "issues": issues
    }))
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
                    {"id": 99, "body": "second", "user": {"login": "codex"}}
                ]
            }
        });

        let filtered =
            apply_result_filter(&value, Some(&ResultFilter::comment_ids(vec![99]))).unwrap();
        let shaped = shape_output(
            &filtered,
            &Cli {
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
                        {"id": 99, "user": {"login": "codex"}}
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
