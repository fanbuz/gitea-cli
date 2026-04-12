use gitea_cli::output::normalize_tool_result;

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
