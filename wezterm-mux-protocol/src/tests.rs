use super::*;
use chrono::TimeZone;
use std::sync::Arc;
use url::Url;

#[test]
fn client_info_json_shape_is_stable() {
    let client_id = Arc::new(ClientId {
        hostname: "host".to_string(),
        username: "user".to_string(),
        pid: 123,
        epoch: 456,
        id: 7,
        ssh_auth_sock: Some("/tmp/ssh.sock".to_string()),
    });
    let info = ClientInfo {
        client_id,
        connected_at: chrono::Utc.timestamp_opt(1, 0).single().unwrap(),
        active_workspace: Some("main".to_string()),
        last_input: chrono::Utc.timestamp_opt(2, 0).single().unwrap(),
        focused_pane_id: Some(9),
    };

    assert_eq!(
        serde_json::to_string(&info).unwrap(),
        concat!(
            "{\"client_id\":{\"hostname\":\"host\",\"username\":\"user\",",
            "\"pid\":123,\"epoch\":456,\"id\":7,\"ssh_auth_sock\":\"/tmp/ssh.sock\"},",
            "\"connected_at\":1,\"active_workspace\":\"main\",\"last_input\":2,",
            "\"focused_pane_id\":9}"
        )
    );
}

#[test]
fn pattern_and_split_json_shape_are_stable() {
    assert_eq!(serde_json::to_string(&Pattern::Regex("foo".to_string())).unwrap(), r#"{"Regex":"foo"}"#);
    assert_eq!(
        serde_json::to_string(&SplitRequest {
            direction: SplitDirection::Vertical,
            target_is_second: false,
            top_level: true,
            size: SplitSize::Cells(12),
        })
        .unwrap(),
        concat!(
            "{\"direction\":\"Vertical\",\"target_is_second\":false,",
            "\"top_level\":true,\"size\":{\"Cells\":12}}"
        )
    );
    assert_eq!(
        serde_json::to_string(&SerdeUrl {
            url: Url::parse("https://example.com/path").unwrap(),
        })
        .unwrap(),
        r#""https://example.com/path""#
    );
}

#[test]
fn pane_node_round_trip_preserves_shape() {
    let node = PaneNode::Leaf(PaneEntry {
        window_id: 11,
        tab_id: 12,
        pane_id: 13,
        title: "pane".to_string(),
        size: wezterm_term_api::TerminalSize {
            rows: 24,
            cols: 80,
            pixel_width: 800,
            pixel_height: 600,
            dpi: 96,
        },
        working_dir: Some(Url::parse("https://example.com/").unwrap().into()),
        is_active_pane: true,
        is_zoomed_pane: false,
        workspace: "default".to_string(),
        cursor_pos: StableCursorPosition::default(),
        physical_top: 0,
        top_row: 1,
        left_col: 2,
        tty_name: Some("ttyS0".to_string()),
    });

    let encoded = varbincode::serialize(&node).unwrap();
    let decoded: PaneNode = varbincode::deserialize(encoded.as_slice()).unwrap();

    assert_eq!(decoded, node);
    assert_eq!(decoded.root_size().unwrap().cols, 80);
    assert_eq!(decoded.window_and_tab_ids(), Some((11, 12)));
}

#[test]
fn renderable_dimensions_round_trip() {
    let dims = RenderableDimensions {
        cols: 80,
        viewport_rows: 24,
        scrollback_rows: 120,
        physical_top: 96,
        scrollback_top: 12,
        dpi: 144,
        pixel_width: 1600,
        pixel_height: 900,
        reverse_video: true,
    };

    let encoded = varbincode::serialize(&dims).unwrap();
    let decoded: RenderableDimensions = varbincode::deserialize(encoded.as_slice()).unwrap();

    assert_eq!(decoded, dims);
}
