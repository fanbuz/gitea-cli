use gitea_cli::output::{filter_comments_by_ids, normalize_tool_result, select_fields};

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

#[test]
fn filter_comments_keeps_only_requested_ids() {
    let value = serde_json::json!([
        {"id": 88, "body": "first"},
        {"id": 99, "body": "second"},
        {"id": 100, "body": "third"}
    ]);

    let filtered = filter_comments_by_ids(&value, &[99, 88]).unwrap();

    assert_eq!(
        filtered,
        serde_json::json!([
            {"id": 88, "body": "first"},
            {"id": 99, "body": "second"}
        ])
    );
}

#[test]
fn filter_comments_returns_empty_array_when_no_ids_match() {
    let value = serde_json::json!([
        {"id": 88, "body": "first"}
    ]);

    let filtered = filter_comments_by_ids(&value, &[999]).unwrap();

    assert_eq!(filtered, serde_json::json!([]));
}

#[test]
fn filter_comments_deduplicates_requested_ids_by_membership() {
    let value = serde_json::json!([
        {"id": 88, "body": "first"},
        {"id": 99, "body": "second"}
    ]);

    let filtered = filter_comments_by_ids(&value, &[88, 88, 99]).unwrap();

    assert_eq!(
        filtered,
        serde_json::json!([
            {"id": 88, "body": "first"},
            {"id": 99, "body": "second"}
        ])
    );
}
