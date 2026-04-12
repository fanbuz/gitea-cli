use std::io::Cursor;

use gitea_cli::mcp::{decode_message, encode_message};

#[test]
fn encode_message_uses_json_lines_frame() {
    let raw = encode_message(&serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    }));

    assert_eq!(
        raw,
        b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\",\"params\":{}}\n"
    );
}

#[test]
fn decode_message_reads_one_json_line() {
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": { "tools": [{ "name": "get_me" }] }
    });
    let raw = encode_message(&payload);
    let mut cursor = Cursor::new(raw);

    let decoded = decode_message(&mut cursor).unwrap();

    assert_eq!(decoded, payload);
}
