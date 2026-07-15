#[cfg(test)]
mod test {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_frame() {
        let mut encoded = Vec::new();
        encode_raw(0x81, 0x42, b"hello", false, &mut encoded).unwrap();
        assert_eq!(&encoded, b"\x08\x42\x81\x01hello");
        let decoded = decode_raw(encoded.as_slice()).unwrap();
        assert_eq!(decoded.ident, 0x81);
        assert_eq!(decoded.serial, 0x42);
        assert_eq!(decoded.data, b"hello");
    }

    #[test]
    fn test_frame_lengths() {
        let mut serial = 1;
        for target_len in &[128, 247, 256, 65536, 16777216] {
            let mut payload = Vec::with_capacity(*target_len);
            payload.resize(*target_len, b'a');
            let mut encoded = Vec::new();
            encode_raw(0x42, serial, payload.as_slice(), false, &mut encoded).unwrap();
            let decoded = decode_raw(encoded.as_slice()).unwrap();
            assert_eq!(decoded.ident, 0x42);
            assert_eq!(decoded.serial, serial);
            assert_eq!(decoded.data, payload);
            serial += 1;
        }
    }

    #[test]
    fn test_pdu_ping() {
        let mut encoded = Vec::new();
        Pdu::Ping(Ping {}).encode(&mut encoded, 0x40).unwrap();
        assert_eq!(&encoded, &[2, 0x40, 1]);
        assert_eq!(
            DecodedPdu {
                serial: 0x40,
                pdu: Pdu::Ping(Ping {})
            },
            Pdu::decode(encoded.as_slice()).unwrap()
        );
    }

    #[test]
    fn stream_decode() {
        let mut encoded = Vec::new();
        Pdu::Ping(Ping {}).encode(&mut encoded, 0x1).unwrap();
        Pdu::Pong(Pong {}).encode(&mut encoded, 0x2).unwrap();
        assert_eq!(encoded.len(), 6);

        let mut cursor = Cursor::new(encoded.as_slice());
        let mut read_buffer = Vec::new();

        assert_eq!(
            Pdu::try_read_and_decode(&mut cursor, &mut read_buffer).unwrap(),
            Some(DecodedPdu {
                serial: 1,
                pdu: Pdu::Ping(Ping {})
            })
        );
        assert_eq!(
            Pdu::try_read_and_decode(&mut cursor, &mut read_buffer).unwrap(),
            Some(DecodedPdu {
                serial: 2,
                pdu: Pdu::Pong(Pong {})
            })
        );
        let err = Pdu::try_read_and_decode(&mut cursor, &mut read_buffer).unwrap_err();
        assert_eq!(
            err.downcast_ref::<std::io::Error>().unwrap().kind(),
            std::io::ErrorKind::UnexpectedEof
        );
    }

    #[test]
    fn test_pdu_ping_base91() {
        let mut encoded = Vec::new();
        {
            let mut encoder = base91::Base91Encoder::new(&mut encoded);
            Pdu::Ping(Ping {}).encode(&mut encoder, 0x41).unwrap();
        }
        assert_eq!(&encoded, &[60, 67, 75, 65]);
        let decoded = base91::decode(&encoded);
        assert_eq!(
            DecodedPdu {
                serial: 0x41,
                pdu: Pdu::Ping(Ping {})
            },
            Pdu::decode(decoded.as_slice()).unwrap()
        );
    }

    #[test]
    fn test_pdu_pong() {
        let mut encoded = Vec::new();
        Pdu::Pong(Pong {}).encode(&mut encoded, 0x42).unwrap();
        assert_eq!(&encoded, &[2, 0x42, 2]);
        assert_eq!(
            DecodedPdu {
                serial: 0x42,
                pdu: Pdu::Pong(Pong {})
            },
            Pdu::decode(encoded.as_slice()).unwrap()
        );
    }

    #[test]
    fn test_bogus_pdu() {
        let mut encoded = Vec::new();
        encode_raw(0xdeadbeef, 0x42, b"hello", false, &mut encoded).unwrap();
        assert_eq!(
            DecodedPdu {
                serial: 0x42,
                pdu: Pdu::Invalid { ident: 0xdeadbeef }
            },
            Pdu::decode(encoded.as_slice()).unwrap()
        );
    }

    #[test]
    fn protocol_client_and_search_pdus_round_trip() {
        let client = ClientInfo {
            client_id: std::sync::Arc::new(ClientId {
                hostname: "host".to_string(),
                username: "user".to_string(),
                pid: 123,
                epoch: 456,
                id: 7,
                ssh_auth_sock: Some("/tmp/ssh.sock".to_string()),
            }),
            connected_at: chrono::Utc.timestamp_opt(10, 0).single().unwrap(),
            active_workspace: Some("main".to_string()),
            last_input: chrono::Utc.timestamp_opt(20, 0).single().unwrap(),
            focused_pane_id: Some(99),
        };

        for pdu in [
            Pdu::GetClientListResponse(GetClientListResponse {
                clients: vec![client.clone()],
            }),
            Pdu::SearchScrollbackRequest(SearchScrollbackRequest {
                pane_id: 99,
                pattern: Pattern::Regex("needle".to_string()),
                range: 10..20,
                limit: Some(5),
            }),
            Pdu::SearchScrollbackResponse(SearchScrollbackResponse {
                results: vec![SearchResult {
                    start_y: 10,
                    start_x: 4,
                    end_y: 10,
                    end_x: 10,
                    match_id: 1,
                }],
            }),
        ] {
            let mut encoded = Vec::new();
            pdu.encode(&mut encoded, 0x44).unwrap();
            assert_eq!(Pdu::decode(encoded.as_slice()).unwrap().pdu, pdu);
        }
    }

    #[test]
    fn protocol_pane_tree_pdu_round_trips() {
        let pdu = Pdu::ListPanesResponse(ListPanesResponse {
            tabs: vec![PaneNode::Leaf(wezterm_mux_protocol::tab::PaneEntry {
                window_id: 1,
                tab_id: 2,
                pane_id: 3,
                title: "pane".to_string(),
                size: TerminalSize {
                    rows: 24,
                    cols: 80,
                    pixel_width: 800,
                    pixel_height: 600,
                    dpi: 96,
                },
                working_dir: Some(std::convert::TryFrom::try_from("https://example.com/".to_string()).unwrap()),
                is_active_pane: true,
                is_zoomed_pane: false,
                workspace: "default".to_string(),
                cursor_pos: StableCursorPosition::default(),
                physical_top: 0,
                top_row: 1,
                left_col: 2,
                tty_name: Some("ttyS0".to_string()),
            })],
            tab_titles: vec!["tab".to_string()],
            window_titles: std::collections::HashMap::from([(1, "window".to_string())]),
        });

        let mut encoded = Vec::new();
        pdu.encode(&mut encoded, 0x45).unwrap();
        assert_eq!(Pdu::decode(encoded.as_slice()).unwrap().pdu, pdu);
    }
}
