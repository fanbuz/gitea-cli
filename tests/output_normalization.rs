use gitea_cli::output::{normalize_tool_result, select_fields};

#[test]
fn parses_single_json_text_content() {
    let raw = serde_json::json!({
        "content": [{
            "type": "text",
            "text": "{\"login\":\"fanpeng\",\"id\":1}"
        }]
    });

    let normalized = normalize_tool_result(raw);

    assert_eq!(
        normalized["parsed"],
        serde_json::json!({"login": "fanpeng", "id": 1})
    );
}

#[test]
fn select_fields_keeps_requested_nested_paths() {
    let value = serde_json::json!({
        "ok": true,
        "kind": "tool_call",
        "result": {
            "parsed": {
                "id": 1,
                "title": "Fix bug",
                "state": "open"
            }
        }
    });

    let selected = select_fields(
        &value,
        &[
            "kind".to_string(),
            "result.parsed.id".to_string(),
            "result.parsed.title".to_string(),
        ],
    )
    .unwrap();

    assert_eq!(
        selected,
        serde_json::json!({
            "kind": "tool_call",
            "result": {
                "parsed": {
                    "id": 1,
                    "title": "Fix bug"
                }
            }
        })
    );
}

#[test]
fn select_fields_ignores_missing_paths() {
    let value = serde_json::json!({
        "ok": true,
        "kind": "doctor",
        "cli": {
            "version": "0.0.6"
        }
    });

    let selected = select_fields(
        &value,
        &[
            "kind".to_string(),
            "cli.name".to_string(),
            "missing.value".to_string(),
        ],
    )
    .unwrap();

    assert_eq!(
        selected,
        serde_json::json!({
            "kind": "doctor"
        })
    );
}

#[test]
fn select_fields_supports_array_index_paths() {
    let value = serde_json::json!({
        "result": {
            "parsed": [
                {"id": 1, "title": "first"},
                {"id": 2, "title": "second"}
            ]
        }
    });

    let selected = select_fields(
        &value,
        &[
            "result.parsed.0.id".to_string(),
            "result.parsed.1.title".to_string(),
        ],
    )
    .unwrap();

    assert_eq!(
        selected,
        serde_json::json!({
            "result": {
                "parsed": [
                    {"id": 1},
                    {"title": "second"}
                ]
            }
        })
    );
}
