use serde_json::Value;

pub fn normalize_tool_result(raw: Value) -> Value {
    let Some(object) = raw.as_object() else {
        return raw;
    };

    let Some(content) = object.get("content").and_then(Value::as_array) else {
        return raw;
    };

    if content.len() != 1 {
        return raw;
    }

    let Some(text_item) = content.first() else {
        return raw;
    };

    let Some(item_type) = text_item.get("type").and_then(Value::as_str) else {
        return raw;
    };

    if item_type != "text" {
        return raw;
    }

    let Some(text) = text_item.get("text").and_then(Value::as_str) else {
        return raw;
    };

    let Ok(parsed) = serde_json::from_str::<Value>(text) else {
        return raw;
    };

    let mut normalized = raw;
    if let Some(object) = normalized.as_object_mut() {
        object.insert("parsed".to_string(), parsed);
    }
    normalized
}
