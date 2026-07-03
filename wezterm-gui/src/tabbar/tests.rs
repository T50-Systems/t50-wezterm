#[cfg(test)]
mod pane_label_tests {
    use super::PaneLabelFormatter;
    use termwiz::cell::unicode_column_width;

    #[test]
    fn normalizes_empty_title_to_shell() {
        let formatter = PaneLabelFormatter::new(false);

        assert_eq!(formatter.title_for("   \t  "), "shell");
        assert_eq!(formatter.label_for(0, ""), " 1:shell ");
    }

    #[test]
    fn keeps_basename_and_collapses_whitespace() {
        let formatter = PaneLabelFormatter::new(false);

        assert_eq!(
            formatter.title_for(r"C:\Users\me\project   shell.exe"),
            "project she…"
        );
        assert_eq!(formatter.title_for("/tmp/my    app"), "my app");
    }

    #[test]
    fn supports_zero_based_indices() {
        let one_based = PaneLabelFormatter::new(false);
        let zero_based = PaneLabelFormatter::new(true);

        assert_eq!(one_based.label_for(2, "pwsh"), " 3:pwsh ");
        assert_eq!(zero_based.label_for(2, "pwsh"), " 2:pwsh ");
    }

    #[test]
    fn truncates_without_exceeding_cell_width() {
        let formatter = PaneLabelFormatter::new(false);
        let title = formatter.title_for("abcdefghijklmnopqrstuvwxyz");

        assert_eq!(title, "abcdefghijk…");
        assert!(unicode_column_width(&title, None) <= 12);
    }

    #[test]
    fn truncates_cjk_by_cell_width() {
        let formatter = PaneLabelFormatter::new(false);
        let title = formatter.title_for("界界界界界界界");

        assert_eq!(title, "界界界界界…");
        assert!(unicode_column_width(&title, None) <= 12);
    }

    #[test]
    fn truncates_emoji_without_splitting_graphemes() {
        let formatter = PaneLabelFormatter::new(false);
        let title = formatter.title_for("😀😀😀😀😀😀😀");

        assert!(title.ends_with('…'));
        assert!(unicode_column_width(&title, None) <= 12);
    }
}

#[cfg(test)]
mod tab_bar_policy_tests {
    use super::{
        IntegratedTitleButtonReservation, StatusLineLayout, StatusLinePlan, TabWidthPolicy,
    };

    #[test]
    fn tab_width_policy_uses_full_width_when_titles_fit() {
        let policy = TabWidthPolicy {
            title_width: 80,
            titles_len: 20,
            number_of_tabs: 3,
            new_tab_len: 2,
            use_fancy_tab_bar: false,
            tab_max_width: 30,
        };

        assert_eq!(policy.max_width(), 30);
    }

    #[test]
    fn tab_width_policy_balances_tabs_when_titles_do_not_fit() {
        let policy = TabWidthPolicy {
            title_width: 20,
            titles_len: 100,
            number_of_tabs: 3,
            new_tab_len: 2,
            use_fancy_tab_bar: false,
            tab_max_width: 30,
        };

        assert_eq!(policy.max_width(), 5);
    }

    #[test]
    fn right_title_button_reservation_uses_numeric_widths_only_when_enabled() {
        let reservation = IntegratedTitleButtonReservation {
            title_width: 80,
            reserve: true,
        };

        assert_eq!(reservation.title_width_after_reservation(&[2, 4]), 74);

        let disabled = IntegratedTitleButtonReservation {
            title_width: 80,
            reserve: false,
        };

        assert_eq!(disabled.title_width_after_reservation(&[2, 4]), 80);
    }

    #[test]
    fn status_line_layout_plans_center_before_right_status() {
        let plan = StatusLineLayout {
            title_width: 10,
            current_x: 2,
        }
        .center_and_right_plan(3);

        assert_eq!(
            plan,
            StatusLinePlan {
                center_width: 5,
                right_width: 3,
                right_trim_left: 0,
            }
        );
    }

    #[test]
    fn status_line_layout_plans_right_status_left_trim() {
        let plan = StatusLineLayout {
            title_width: 5,
            current_x: 2,
        }
        .right_only_plan(5);

        assert_eq!(
            plan,
            StatusLinePlan {
                center_width: 0,
                right_width: 3,
                right_trim_left: 2,
            }
        );
    }
}

#[cfg(test)]
mod tab_bar_constructor_tests {
    use super::{TabBarItem, TabBarState};
    use crate::termwindow::{PaneInformation, TabInformation};
    use config::ConfigHandle;

    #[test]
    fn primary_constructor_does_not_emit_center_status_without_center_input() {
        let config = ConfigHandle::default_config();
        let tabs: Vec<TabInformation> = vec![];
        let panes: Vec<PaneInformation> = vec![];
        let tab_bar =
            TabBarState::new_primary(80, None, &tabs, &panes, None, &config, "LEFT", "RIGHT");

        assert!(tab_bar
            .items()
            .iter()
            .any(|entry| entry.item == TabBarItem::LeftStatus));
        assert!(tab_bar
            .items()
            .iter()
            .any(|entry| entry.item == TabBarItem::RightStatus));
        assert!(!tab_bar
            .items()
            .iter()
            .any(|entry| entry.item == TabBarItem::CenterStatus));
        assert!(!tab_bar
            .items()
            .iter()
            .any(|entry| matches!(entry.item, TabBarItem::PaneStatus { .. })));
    }

    #[test]
    fn status_bar_constructor_never_emits_activation_items() {
        let config = ConfigHandle::default_config();
        let tabs: Vec<TabInformation> = vec![];
        let panes: Vec<PaneInformation> = vec![];
        let tab_bar = TabBarState::new_status_bar(
            80, &tabs, &panes, None, &config, "LEFT", "CENTER", "RIGHT",
        );

        assert!(tab_bar
            .items()
            .iter()
            .any(|entry| entry.item == TabBarItem::LeftStatus));
        assert!(tab_bar
            .items()
            .iter()
            .any(|entry| entry.item == TabBarItem::CenterStatus));
        assert!(tab_bar
            .items()
            .iter()
            .any(|entry| entry.item == TabBarItem::RightStatus));
        assert!(!tab_bar
            .items()
            .iter()
            .any(|entry| matches!(entry.item, TabBarItem::Tab { .. })));
        assert!(!tab_bar
            .items()
            .iter()
            .any(|entry| entry.item == TabBarItem::NewTabButton));
    }
}

#[cfg(test)]
mod ui_item_geometry_tests {
    use super::{TabBarItem, TabBarState, TabEntry};
    use termwiz::surface::SEQ_ZERO;
    use wezterm_term::Line;

    fn tab_bar_with_pane_status() -> TabBarState {
        TabBarState {
            line: Line::with_width(0, SEQ_ZERO),
            items: vec![TabEntry {
                item: TabBarItem::PaneStatus {
                    pane_id: 42,
                    active: true,
                },
                title: Line::with_width(0, SEQ_ZERO),
                x: 3,
                width: 5,
            }],
        }
    }

    #[test]
    fn compute_ui_items_preserves_pane_status_cell_geometry() {
        let ui_items = tab_bar_with_pane_status().compute_ui_items(10, 20, 8);

        assert_eq!(ui_items.len(), 1);
        assert_eq!(ui_items[0].x, 24);
        assert_eq!(ui_items[0].y, 10);
        assert_eq!(ui_items[0].width, 40);
        assert_eq!(ui_items[0].height, 20);
        assert_eq!(
            ui_items[0].item_type,
            crate::termwindow::UIItemType::TabBar(TabBarItem::PaneStatus {
                pane_id: 42,
                active: true,
            })
        );
    }

    #[test]
    fn compute_ui_items_uses_supplied_secondary_bar_y_coordinate() {
        let secondary_y = 30;
        let ui_items = tab_bar_with_pane_status().compute_ui_items(secondary_y, 20, 8);

        assert_eq!(ui_items[0].y, secondary_y);
    }
}
