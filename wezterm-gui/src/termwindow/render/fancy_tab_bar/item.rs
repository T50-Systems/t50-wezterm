impl crate::TermWindow {
    fn fancy_tab_item_to_element(
        &self,
        font: &Rc<LoadedFont>,
        metrics: &RenderMetrics,
        palette: &ColorPalette,
        colors: &TabBarColors,
        bar_colors: &ElementColors,
        item: &TabEntry,
    ) -> Element {
        let element = Element::with_line(&font, &item.title, palette);

        let bg_color = item
            .title
            .get_cell(0)
            .and_then(|c| match c.attrs().background() {
                ColorAttribute::Default => None,
                col => Some(palette.resolve_bg(col)),
            });
        let fg_color = item
            .title
            .get_cell(0)
            .and_then(|c| match c.attrs().foreground() {
                ColorAttribute::Default => None,
                col => Some(palette.resolve_fg(col)),
            });

        let new_tab = colors.new_tab();
        let new_tab_hover = colors.new_tab_hover();
        let active_tab = colors.active_tab();

        match item.item {
            TabBarItem::LeftStatus => element
                .item_type(UIItemType::TabBar(TabBarItem::None))
                .line_height(Some(1.75))
                .margin(BoxDimension {
                    left: Dimension::Cells(0.),
                    right: Dimension::Cells(0.),
                    top: Dimension::Cells(0.0),
                    bottom: Dimension::Cells(0.),
                })
                .padding(BoxDimension {
                    left: Dimension::Cells(0.5),
                    right: Dimension::Cells(0.),
                    top: Dimension::Cells(0.),
                    bottom: Dimension::Cells(0.),
                })
                .border(BoxDimension::new(Dimension::Pixels(0.)))
                .colors(bar_colors.clone()),
            TabBarItem::RightStatus => element
                .item_type(UIItemType::TabBar(TabBarItem::None))
                .line_height(Some(1.75))
                .float(Float::Right)
                .margin(BoxDimension {
                    left: Dimension::Cells(0.),
                    right: Dimension::Cells(0.),
                    top: Dimension::Cells(0.0),
                    bottom: Dimension::Cells(0.),
                })
                .padding(BoxDimension {
                    left: Dimension::Cells(0.),
                    right: Dimension::Cells(0.5),
                    top: Dimension::Cells(0.),
                    bottom: Dimension::Cells(0.),
                })
                .border(BoxDimension::new(Dimension::Pixels(0.)))
                .colors(bar_colors.clone()),
            TabBarItem::PaneStatus { active, .. } => element
                .item_type(UIItemType::TabBar(item.item))
                .line_height(Some(1.75))
                .margin(BoxDimension {
                    left: Dimension::Cells(0.),
                    right: Dimension::Cells(0.),
                    top: Dimension::Cells(0.0),
                    bottom: Dimension::Cells(0.),
                })
                .padding(BoxDimension {
                    left: Dimension::Cells(0.5),
                    right: Dimension::Cells(0.5),
                    top: Dimension::Cells(0.),
                    bottom: Dimension::Cells(0.),
                })
                .border(BoxDimension::new(Dimension::Pixels(1.0)))
                .colors(if active {
                    ElementColors {
                        border: BorderColor::new(active_tab.bg_color.to_linear()),
                        bg: active_tab.bg_color.to_linear().into(),
                        text: active_tab.fg_color.to_linear().into(),
                    }
                } else {
                    ElementColors {
                        border: BorderColor::new(colors.inactive_tab().bg_color.to_linear()),
                        bg: colors.inactive_tab().bg_color.to_linear().into(),
                        text: colors.inactive_tab().fg_color.to_linear().into(),
                    }
                })
                .hover_colors(Some(ElementColors {
                    border: BorderColor::new(colors.inactive_tab_hover().bg_color.to_linear()),
                    bg: colors.inactive_tab_hover().bg_color.to_linear().into(),
                    text: colors.inactive_tab_hover().fg_color.to_linear().into(),
                })),
            TabBarItem::CenterStatus => element
                .item_type(UIItemType::TabBar(TabBarItem::None))
                .line_height(Some(1.75))
                .display(DisplayType::Block)
                .margin(BoxDimension {
                    left: Dimension::Cells(0.),
                    right: Dimension::Cells(0.),
                    top: Dimension::Cells(0.0),
                    bottom: Dimension::Cells(0.),
                })
                .padding(BoxDimension {
                    left: Dimension::Cells(0.),
                    right: Dimension::Cells(0.),
                    top: Dimension::Cells(0.),
                    bottom: Dimension::Cells(0.),
                })
                .border(BoxDimension::new(Dimension::Pixels(0.)))
                .colors(ElementColors {
                    border: BorderColor::new(colors.inactive_tab().bg_color.to_linear()),
                    bg: colors.inactive_tab().bg_color.to_linear().into(),
                    text: colors.inactive_tab().fg_color.to_linear().into(),
                })
                .hover_colors(Some(ElementColors {
                    border: BorderColor::new(colors.inactive_tab_hover().bg_color.to_linear()),
                    bg: colors.inactive_tab_hover().bg_color.to_linear().into(),
                    text: colors.inactive_tab_hover().fg_color.to_linear().into(),
                })),
            TabBarItem::None => {
                unreachable!("status items are handled via TabBarZone")
            }
            TabBarItem::NewTabButton => Element::new(
                &font,
                ElementContent::Poly {
                    line_width: metrics.underline_height.max(2),
                    poly: SizedPoly {
                        poly: PLUS_BUTTON,
                        width: Dimension::Pixels(metrics.cell_size.height as f32 / 2.),
                        height: Dimension::Pixels(metrics.cell_size.height as f32 / 2.),
                    },
                },
            )
            .vertical_align(VerticalAlign::Middle)
            .item_type(UIItemType::TabBar(item.item.clone()))
            .margin(BoxDimension {
                left: Dimension::Cells(0.5),
                right: Dimension::Cells(0.),
                top: Dimension::Cells(0.2),
                bottom: Dimension::Cells(0.),
            })
            .padding(BoxDimension {
                left: Dimension::Cells(0.5),
                right: Dimension::Cells(0.5),
                top: Dimension::Cells(0.2),
                bottom: Dimension::Cells(0.25),
            })
            .border(BoxDimension::new(Dimension::Pixels(1.)))
            .colors(ElementColors {
                border: BorderColor::default(),
                bg: new_tab.bg_color.to_linear().into(),
                text: new_tab.fg_color.to_linear().into(),
            })
            .hover_colors(Some(ElementColors {
                border: BorderColor::default(),
                bg: new_tab_hover.bg_color.to_linear().into(),
                text: new_tab_hover.fg_color.to_linear().into(),
            })),
            TabBarItem::Tab { active, .. } if active => element
                .vertical_align(VerticalAlign::Bottom)
                .item_type(UIItemType::TabBar(item.item.clone()))
                .margin(BoxDimension {
                    left: Dimension::Cells(0.),
                    right: Dimension::Cells(0.),
                    top: Dimension::Cells(0.2),
                    bottom: Dimension::Cells(0.),
                })
                .padding(BoxDimension {
                    left: Dimension::Cells(0.5),
                    right: Dimension::Cells(0.5),
                    top: Dimension::Cells(0.2),
                    bottom: Dimension::Cells(0.25),
                })
                .border(BoxDimension::new(Dimension::Pixels(1.)))
                .border_corners(Some(Corners {
                    top_left: SizedPoly {
                        width: Dimension::Cells(0.5),
                        height: Dimension::Cells(0.5),
                        poly: TOP_LEFT_ROUNDED_CORNER,
                    },
                    top_right: SizedPoly {
                        width: Dimension::Cells(0.5),
                        height: Dimension::Cells(0.5),
                        poly: TOP_RIGHT_ROUNDED_CORNER,
                    },
                    bottom_left: SizedPoly::none(),
                    bottom_right: SizedPoly::none(),
                }))
                .colors(ElementColors {
                    border: BorderColor::new(
                        bg_color
                            .unwrap_or_else(|| active_tab.bg_color.into())
                            .to_linear(),
                    ),
                    bg: bg_color
                        .unwrap_or_else(|| active_tab.bg_color.into())
                        .to_linear()
                        .into(),
                    text: fg_color
                        .unwrap_or_else(|| active_tab.fg_color.into())
                        .to_linear()
                        .into(),
                }),
            TabBarItem::Tab { .. } => element
                .vertical_align(VerticalAlign::Bottom)
                .item_type(UIItemType::TabBar(item.item.clone()))
                .margin(BoxDimension {
                    left: Dimension::Cells(0.),
                    right: Dimension::Cells(0.),
                    top: Dimension::Cells(0.2),
                    bottom: Dimension::Cells(0.),
                })
                .padding(BoxDimension {
                    left: Dimension::Cells(0.5),
                    right: Dimension::Cells(0.5),
                    top: Dimension::Cells(0.2),
                    bottom: Dimension::Cells(0.25),
                })
                .border(BoxDimension::new(Dimension::Pixels(1.)))
                .border_corners(Some(Corners {
                    top_left: SizedPoly {
                        width: Dimension::Cells(0.5),
                        height: Dimension::Cells(0.5),
                        poly: TOP_LEFT_ROUNDED_CORNER,
                    },
                    top_right: SizedPoly {
                        width: Dimension::Cells(0.5),
                        height: Dimension::Cells(0.5),
                        poly: TOP_RIGHT_ROUNDED_CORNER,
                    },
                    bottom_left: SizedPoly {
                        width: Dimension::Cells(0.),
                        height: Dimension::Cells(0.33),
                        poly: &[],
                    },
                    bottom_right: SizedPoly {
                        width: Dimension::Cells(0.),
                        height: Dimension::Cells(0.33),
                        poly: &[],
                    },
                }))
                .colors({
                    let inactive_tab = colors.inactive_tab();
                    let bg = bg_color
                        .unwrap_or_else(|| inactive_tab.bg_color.into())
                        .to_linear();
                    let edge = colors.inactive_tab_edge().to_linear();
                    ElementColors {
                        border: BorderColor {
                            left: bg,
                            right: edge,
                            top: bg,
                            bottom: bg,
                        },
                        bg: bg.into(),
                        text: fg_color
                            .unwrap_or_else(|| inactive_tab.fg_color.into())
                            .to_linear()
                            .into(),
                    }
                })
                .hover_colors({
                    let inactive_tab_hover = colors.inactive_tab_hover();
                    Some(ElementColors {
                        border: BorderColor::new(
                            bg_color
                                .unwrap_or_else(|| inactive_tab_hover.bg_color.into())
                                .to_linear(),
                        ),
                        bg: bg_color
                            .unwrap_or_else(|| inactive_tab_hover.bg_color.into())
                            .to_linear()
                            .into(),
                        text: fg_color
                            .unwrap_or_else(|| inactive_tab_hover.fg_color.into())
                            .to_linear()
                            .into(),
                    })
                }),
            TabBarItem::WindowButton(button) => window_button_element(
                button,
                self.window_state.contains(window::WindowState::MAXIMIZED),
                &font,
                &metrics,
                &self.config,
            ),
        }
    }
}
