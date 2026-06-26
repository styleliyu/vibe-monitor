use serde_json::json;
use tempfile::TempDir;
use vibe_monitor_lib::{
    attention::list_attention_items,
    codex::{
        detect_with_runner,
        jsonrpc::{encode_request, JsonRpcRequest},
        notification_to_attention_item, notification_value_to_attention_item, CodexApprovalRequest,
    },
    db,
};

#[test]
fn codex_detect_reports_unavailable_when_version_command_fails() {
    let availability = detect_with_runner(|| Err("Access is denied".to_string()));

    assert!(!availability.available);
    assert_eq!(availability.version, None);
    assert_eq!(availability.error, Some("Access is denied".to_string()));
}

#[test]
fn approval_notification_value_creates_attention_item_when_workspace_is_present() {
    let temp_dir = TempDir::new().expect("temp dir");
    db::init_db(temp_dir.path()).expect("db init");
    let payload = json!({
        "method": "approval/requested",
        "params": {
            "workspaceId": "workspace-1",
            "threadId": "thread-1",
            "approvalId": "approval-1",
            "title": "Run command",
            "summary": "codex wants to run npm test"
        }
    });

    let item = notification_value_to_attention_item(temp_dir.path(), &payload)
        .expect("mapped item")
        .expect("approval should create an item");

    assert_eq!(item.workspace_id, "workspace-1");
    assert_eq!(item.kind, "approval");
    assert_eq!(item.priority, 3);
}

#[test]
fn jsonrpc_request_omits_jsonrpc_version_field() {
    let request = JsonRpcRequest {
        id: 7,
        method: "thread/list".to_string(),
        params: json!({ "cwd": "D:\\AIdeas\\CodingMonitor" }),
    };

    let encoded = encode_request(&request).expect("request should serialize");
    let value: serde_json::Value = serde_json::from_str(&encoded).expect("valid json");

    assert_eq!(value["id"], 7);
    assert_eq!(value["method"], "thread/list");
    assert!(value.get("jsonrpc").is_none());
}

#[test]
fn approval_notification_creates_high_priority_attention_item() {
    let temp_dir = TempDir::new().expect("temp dir");
    db::init_db(temp_dir.path()).expect("db init");

    let item = notification_to_attention_item(
        temp_dir.path(),
        "workspace-1",
        CodexApprovalRequest {
            thread_id: "thread-1".to_string(),
            approval_id: "approval-1".to_string(),
            title: "Run command".to_string(),
            summary: "codex wants to run npm test".to_string(),
        },
    )
    .expect("attention item")
    .expect("approval should create an item");

    assert_eq!(item.kind, "approval");
    assert_eq!(item.priority, 3);
    assert_eq!(item.title, "Codex approval required");
    assert_eq!(item.session_id, Some("thread-1".to_string()));
    assert_eq!(
        item.action_ref,
        Some("codex://thread/thread-1/approval/approval-1".to_string())
    );

    let active_items =
        list_attention_items(temp_dir.path(), Some("workspace-1".to_string())).expect("items");
    assert_eq!(active_items.len(), 1);
}
