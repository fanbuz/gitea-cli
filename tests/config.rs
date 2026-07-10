use std::{fs, path::PathBuf};

use gitea_cli::config::{load_config_view, load_gitea_server_config, render_init_config};
use tempfile::tempdir;

#[test]
fn loads_inline_token_profile_as_server_env() {
    let dir = tempdir().unwrap();
    let config_path: PathBuf = dir.path().join("config.toml");
    fs::write(
        &config_path,
        r#"
version = 1
active_profile = "local"

[profiles.local]
host = "https://gitea.example.com/"

[profiles.local.mcp]
command = "gitea-mcp-server"
args = ["-t", "stdio"]

[profiles.local.mcp.env]
GITEA_MODE = "test"

[profiles.local.auth]
type = "token"
token = "secret"
"#,
    )
    .unwrap();

    let server = load_gitea_server_config(&config_path, None).unwrap();

    assert_eq!(server.command, "gitea-mcp-server");
    assert_eq!(server.args, vec!["-t", "stdio"]);
    assert_eq!(
        server.env.get("GITEA_HOST").unwrap(),
        "https://gitea.example.com/"
    );
    assert_eq!(server.env.get("GITEA_ACCESS_TOKEN").unwrap(), "secret");
    assert_eq!(server.env.get("GITEA_MODE").unwrap(), "test");
    assert!(server.metadata.auth.inline_token);
    assert_eq!(server.metadata.profile, "local");
    assert_eq!(server.metadata.warnings.len(), 1);
}

#[test]
fn loads_token_file_profile_as_server_env() {
    let dir = tempdir().unwrap();
    let token_path: PathBuf = dir.path().join("token.txt");
    let config_path: PathBuf = dir.path().join("config.toml");
    fs::write(&token_path, "secret").unwrap();
    let token_path_value = token_path
        .display()
        .to_string()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    fs::write(
        &config_path,
        format!(
            r#"
version = 1
active_profile = "file"

[profiles.file]
host = "https://gitea.example.com/"

[profiles.file.mcp]
command = "gitea-mcp-server"
args = ["-t", "stdio"]

[profiles.file.auth]
type = "token_file"
token_file = "{}"
"#,
            token_path_value
        ),
    )
    .unwrap();

    let server = load_gitea_server_config(&config_path, None).unwrap();

    assert_eq!(
        server.env.get("GITEA_ACCESS_TOKEN_FILE").unwrap(),
        &token_path.display().to_string()
    );
    assert!(!server.env.contains_key("GITEA_ACCESS_TOKEN"));
    assert_eq!(
        server.metadata.auth.token_file.as_ref().unwrap(),
        &token_path
    );
}

#[test]
fn config_view_does_not_include_inline_token_value() {
    let dir = tempdir().unwrap();
    let config_path: PathBuf = dir.path().join("config.toml");
    fs::write(
        &config_path,
        r#"
version = 1
active_profile = "local"

[profiles.local]
host = "https://gitea.example.com/"

[profiles.local.mcp]
command = "gitea-mcp-server"
args = ["-t", "stdio"]

[profiles.local.auth]
type = "token"
token = "secret"
"#,
    )
    .unwrap();

    let view = load_config_view(&config_path, None).unwrap();
    let rendered = serde_json::to_string(&view).unwrap();

    assert!(view.auth.inline_token);
    assert!(rendered.contains("inline_token"));
    assert!(!rendered.contains("secret"));
}

#[test]
fn profile_override_selects_requested_profile() {
    let dir = tempdir().unwrap();
    let config_path: PathBuf = dir.path().join("config.toml");
    fs::write(
        &config_path,
        r#"
version = 1
active_profile = "one"

[profiles.one]
host = "https://one.example.com/"

[profiles.one.mcp]
command = "gitea-mcp-server"

[profiles.one.auth]
type = "token"
token = "one"

[profiles.two]
host = "https://two.example.com/"

[profiles.two.mcp]
command = "gitea-mcp-server"

[profiles.two.auth]
type = "token"
token = "two"
"#,
    )
    .unwrap();

    let server = load_gitea_server_config(&config_path, Some("two")).unwrap();

    assert_eq!(server.metadata.profile, "two");
    assert_eq!(
        server.env.get("GITEA_HOST").unwrap(),
        "https://two.example.com/"
    );
    assert_eq!(server.env.get("GITEA_ACCESS_TOKEN").unwrap(), "two");
}

#[test]
fn renders_minimal_init_config() {
    let rendered = render_init_config("default", "https://gitea.example.com/").unwrap();

    assert!(rendered.contains("version = 1"));
    assert!(rendered.contains("active_profile = \"default\""));
    assert!(rendered.contains("host = \"https://gitea.example.com/\""));
    assert!(rendered.contains("token_env = \"GITEA_ACCESS_TOKEN\""));
}
