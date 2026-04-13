use anyhow::{Result, anyhow};
use serde_json::{Map, Value};

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

pub fn select_fields(value: &Value, fields: &[String]) -> Result<Value> {
    let mut selected = Value::Object(Map::new());

    for field in fields {
        let path = parse_field_path(field)?;
        let Some(item) = get_path(value, &path) else {
            continue;
        };
        insert_path(&mut selected, &path, item.clone());
    }

    Ok(selected)
}

fn parse_field_path(field: &str) -> Result<Vec<&str>> {
    if field.trim().is_empty() {
        return Err(anyhow!("--fields 里包含空字段"));
    }

    let segments = field.split('.').collect::<Vec<_>>();
    if segments.iter().any(|segment| segment.is_empty()) {
        return Err(anyhow!("字段路径包含空片段: {field}"));
    }

    Ok(segments)
}

fn get_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;

    for segment in path {
        current = match current {
            Value::Object(object) => object.get(*segment)?,
            Value::Array(items) => {
                let index = segment.parse::<usize>().ok()?;
                items.get(index)?
            }
            _ => return None,
        };
    }

    Some(current)
}

fn insert_path(target: &mut Value, path: &[&str], value: Value) {
    if path.is_empty() {
        *target = value;
        return;
    }

    match path[0].parse::<usize>() {
        Ok(index) => insert_array_path(target, index, &path[1..], value),
        Err(_) => insert_object_path(target, path[0], &path[1..], value),
    }
}

fn insert_object_path(target: &mut Value, key: &str, rest: &[&str], value: Value) {
    if !target.is_object() {
        *target = Value::Object(Map::new());
    }

    let object = target
        .as_object_mut()
        .expect("target should be an object after initialization");

    if rest.is_empty() {
        object.insert(key.to_string(), value);
        return;
    }

    let entry = object
        .entry(key.to_string())
        .or_insert_with(|| empty_container_for(rest[0]));
    insert_path(entry, rest, value);
}

fn insert_array_path(target: &mut Value, index: usize, rest: &[&str], value: Value) {
    if !target.is_array() {
        *target = Value::Array(Vec::new());
    }

    let items = target
        .as_array_mut()
        .expect("target should be an array after initialization");
    while items.len() <= index {
        items.push(Value::Null);
    }

    if rest.is_empty() {
        items[index] = value;
        return;
    }

    if items[index].is_null() {
        items[index] = empty_container_for(rest[0]);
    }

    insert_path(&mut items[index], rest, value);
}

fn empty_container_for(next_segment: &str) -> Value {
    if next_segment.parse::<usize>().is_ok() {
        Value::Array(Vec::new())
    } else {
        Value::Object(Map::new())
    }
}
