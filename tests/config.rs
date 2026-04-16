use std::{fs, path::PathBuf};

use gitea_cli::config::load_gitea_server_config;
use tempfile::tempdir;

#[test]
fn reads_gitea_server_from_codex_config() {
    let dir = tempdir().unwrap();
    let config_path: PathBuf = dir.path().join("config.toml");
    fs::write(
        &config_path,
        r#"
[mcp_servers.gitea]
type = "stdio"
command = "gitea-mcp-server"
args = ["-t", "stdio", "--host", "https://gitea.example.com/", "--token", "secret"]

[mcp_servers.gitea.env]
GITEA_MODE = "codex"
"#,
    )
    .unwrap();

    let server = load_gitea_server_config(&config_path).unwrap();

    assert_eq!(server.command, "gitea-mcp-server");
    assert_eq!(server.args.last().unwrap(), "secret");
    assert_eq!(server.env.get("GITEA_MODE").unwrap(), "codex");
}

#[test]
fn reads_gitea_server_when_other_mcp_servers_use_url_transport() {
    let dir = tempdir().unwrap();
    let config_path: PathBuf = dir.path().join("config.toml");
    fs::write(
        &config_path,
        r#"
[mcp_servers.gitea]
type = "stdio"
command = "gitea-mcp-server"
args = ["-t", "stdio", "--host", "https://gitea.example.com/", "--token", "secret"]

[mcp_servers.idea]
url = "http://127.0.0.1:64342/stream"
"#,
    )
    .unwrap();

    let server = load_gitea_server_config(&config_path).unwrap();

    assert_eq!(server.command, "gitea-mcp-server");
    assert_eq!(server.args.last().unwrap(), "secret");
}
