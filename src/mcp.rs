use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use anyhow::{Context, Result, anyhow, bail};
use serde_json::{Value, json};

use crate::config::GiteaServerConfig;
use crate::output::normalize_tool_result;

pub fn encode_message(payload: &Value) -> Vec<u8> {
    let mut raw = encode_value(payload).into_bytes();
    raw.push(b'\n');
    raw
}

pub fn decode_message<R: BufRead>(reader: &mut R) -> Result<Value> {
    let mut first_line = String::new();

    loop {
        first_line.clear();
        let bytes = reader.read_line(&mut first_line)?;
        if bytes == 0 {
            bail!("读取 MCP 消息失败: 遇到 EOF");
        }

        if first_line.trim().is_empty() {
            continue;
        }

        break;
    }

    let trimmed = first_line.trim();
    if trimmed.starts_with('{') {
        return serde_json::from_str(trimmed).context("解析 JSON Lines MCP 消息失败");
    }

    if !first_line
        .to_ascii_lowercase()
        .starts_with("content-length:")
    {
        bail!("无法识别的 MCP 消息帧: {}", trimmed);
    }

    let (_, length_value) = first_line
        .split_once(':')
        .context("Content-Length 头格式错误")?;
    let content_length: usize = length_value
        .trim()
        .parse()
        .context("Content-Length 不是合法数字")?;

    loop {
        let mut header_line = String::new();
        let bytes = reader.read_line(&mut header_line)?;
        if bytes == 0 {
            bail!("读取 Content-Length 头部时遇到 EOF");
        }
        if header_line.trim().is_empty() {
            break;
        }
    }

    let mut body = vec![0_u8; content_length];
    reader
        .read_exact(&mut body)
        .context("读取 Content-Length 消息体失败")?;
    serde_json::from_slice(&body).context("解析 Content-Length MCP 消息失败")
}

pub struct McpSession {
    child: Child,
    reader: BufReader<ChildStdout>,
    writer: ChildStdin,
    next_id: u64,
}

impl McpSession {
    pub fn start(config: &GiteaServerConfig) -> Result<Self> {
        let mut command = Command::new(&config.command);
        command
            .args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .envs(&config.env);

        let mut child = command
            .spawn()
            .with_context(|| format!("启动 MCP server 失败: {}", config.command))?;
        let stdout = child.stdout.take().context("MCP server stdout 不可用")?;
        let stdin = child.stdin.take().context("MCP server stdin 不可用")?;

        let mut session = Self {
            child,
            reader: BufReader::new(stdout),
            writer: stdin,
            next_id: 1,
        };

        session.initialize()?;
        Ok(session)
    }

    pub fn list_tools(&mut self) -> Result<Value> {
        self.request("tools/list", json!({}))
    }

    pub fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value> {
        let result = self.request(
            "tools/call",
            json!({
                "name": name,
                "arguments": arguments
            }),
        )?;
        Ok(normalize_tool_result(result))
    }

    fn initialize(&mut self) -> Result<()> {
        let _ = self.request(
            "initialize",
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": env!("CARGO_PKG_NAME"),
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
        )?;

        self.notify("notifications/initialized", json!({}))?;
        Ok(())
    }

    fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id;
        self.next_id += 1;

        let payload = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });
        self.send(&payload)?;

        loop {
            let response = decode_message(&mut self.reader)?;
            let Some(response_id) = response.get("id").and_then(Value::as_u64) else {
                continue;
            };

            if response_id != id {
                continue;
            }

            if let Some(error) = response.get("error") {
                bail!("MCP 返回错误: {}", error);
            }

            return response
                .get("result")
                .cloned()
                .ok_or_else(|| anyhow!("MCP 响应缺少 result 字段"));
        }
    }

    fn notify(&mut self, method: &str, params: Value) -> Result<()> {
        let payload = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });
        self.send(&payload)
    }

    fn send(&mut self, payload: &Value) -> Result<()> {
        self.writer
            .write_all(&encode_message(payload))
            .context("写入 MCP 请求失败")?;
        self.writer.flush().context("刷新 MCP 请求失败")
    }
}

impl Drop for McpSession {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn encode_value(value: &Value) -> String {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            serde_json::to_string(value).expect("JSON serialization should not fail")
        }
        Value::Array(items) => {
            let encoded_items = items.iter().map(encode_value).collect::<Vec<_>>();
            format!("[{}]", encoded_items.join(","))
        }
        Value::Object(object) => {
            let preferred_order = ["jsonrpc", "id", "method", "params", "result", "error"];
            let mut parts = Vec::with_capacity(object.len());

            for key in preferred_order {
                if let Some(item) = object.get(key) {
                    parts.push(format!(
                        "{}:{}",
                        serde_json::to_string(key).expect("JSON serialization should not fail"),
                        encode_value(item)
                    ));
                }
            }

            for (key, item) in object {
                if preferred_order.contains(&key.as_str()) {
                    continue;
                }
                parts.push(format!(
                    "{}:{}",
                    serde_json::to_string(key).expect("JSON serialization should not fail"),
                    encode_value(item)
                ));
            }

            format!("{{{}}}", parts.join(","))
        }
    }
}
