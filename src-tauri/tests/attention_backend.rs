use tempfile::tempdir;
use vibe_monitor_lib::{attention, db};

fn create_input(
    workspace_id: &str,
    kind: &str,
    priority: i64,
    title: &str,
) -> attention::CreateAttentionItem {
    attention::CreateAttentionItem {
        workspace_id: workspace_id.to_string(),
        session_id: None,
        kind: kind.to_string(),
        priority,
        title: title.to_string(),
        summary: format!("{title} summary"),
        action_label: None,
        action_ref: None,
    }
}

#[test]
fn attention_create_list_and_resolve_persists_active_queue() {
    let app_data = tempdir().expect("app data tempdir");
    db::init_db(app_data.path()).expect("database initialization");

    let low = attention::create_attention_item(
        app_data.path(),
        create_input("workspace-1", "info", 1, "Low priority"),
    )
    .expect("low priority item");
    let high = attention::create_attention_item(
        app_data.path(),
        create_input("workspace-1", "failed", 3, "High priority"),
    )
    .expect("high priority item");

    let active = attention::list_attention_items(app_data.path(), Some("workspace-1".to_string()))
        .expect("active attention list");
    assert_eq!(
        active
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
        vec![high.id.as_str(), low.id.as_str()]
    );

    attention::resolve_attention_item(app_data.path(), high.id.clone())
        .expect("attention item should resolve");

    let active_after_resolve =
        attention::list_attention_items(app_data.path(), Some("workspace-1".to_string()))
            .expect("active attention list");
    assert_eq!(active_after_resolve, vec![low]);
}

#[test]
fn attention_list_filters_by_workspace_and_excludes_resolved_items() {
    let app_data = tempdir().expect("app data tempdir");
    db::init_db(app_data.path()).expect("database initialization");

    let workspace_one = attention::create_attention_item(
        app_data.path(),
        create_input("workspace-1", "blocked", 3, "Workspace one"),
    )
    .expect("workspace one item");
    let workspace_two = attention::create_attention_item(
        app_data.path(),
        create_input("workspace-2", "approval", 2, "Workspace two"),
    )
    .expect("workspace two item");

    attention::resolve_attention_item(app_data.path(), workspace_two.id)
        .expect("workspace two item should resolve");

    let workspace_one_items =
        attention::list_attention_items(app_data.path(), Some("workspace-1".to_string()))
            .expect("workspace one list");
    assert_eq!(workspace_one_items, vec![workspace_one.clone()]);

    let all_active =
        attention::list_attention_items(app_data.path(), None).expect("all active attention list");
    assert_eq!(all_active, vec![workspace_one]);
}

#[test]
fn attention_create_rejects_invalid_kind_and_priority() {
    let app_data = tempdir().expect("app data tempdir");
    db::init_db(app_data.path()).expect("database initialization");

    let invalid_kind = attention::create_attention_item(
        app_data.path(),
        create_input("workspace-1", "surprise", 1, "Invalid kind"),
    );
    assert!(invalid_kind.is_err());

    let invalid_priority = attention::create_attention_item(
        app_data.path(),
        create_input("workspace-1", "info", 4, "Invalid priority"),
    );
    assert!(invalid_priority.is_err());
}
