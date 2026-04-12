use std::process::Command;

use tempfile::tempdir;

#[test]
fn doctor_json_includes_cli_version() {
    let home = tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_gitea-cli"))
        .arg("--json")
        .arg("doctor")
        .env("HOME", home.path())
        .env("USERPROFILE", home.path())
        .output()
        .unwrap();

    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();
    let value: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(value["kind"], "doctor");
    assert_eq!(value["cli"]["name"], "gitea-cli");
    assert_eq!(value["cli"]["version"], env!("CARGO_PKG_VERSION"));
}
