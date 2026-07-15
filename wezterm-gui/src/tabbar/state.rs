#[derive(Clone, Debug, PartialEq)]
pub struct TabBarState {
    line: Line,
    items: Vec<TabEntry>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabBarItem {
    None,
    LeftStatus,
    CenterStatus,
    RightStatus,
    PaneStatus { pane_id: PaneId, active: bool },
    Tab { tab_idx: usize, active: bool },
    NewTabButton,
    WindowButton(IntegratedTitleButton),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabBarZone {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabBarContentMode {
    Full,
    StatusOnly,
}

impl TabBarItem {
    pub fn zone(self) -> TabBarZone {
        match self {
            Self::LeftStatus => TabBarZone::Left,
            Self::RightStatus => TabBarZone::Right,
            // Center is everything that is neither left nor right status.
            // It includes the main tabbar payload and any center overlays.
            Self::None
            | Self::CenterStatus
            | Self::PaneStatus { .. }
            | Self::Tab { .. }
            | Self::NewTabButton
            | Self::WindowButton(_) => TabBarZone::Center,
        }
    }

    pub fn is_center(self) -> bool {
        self.zone() == TabBarZone::Center
    }

    pub fn is_left_or_right(self) -> bool {
        let zone = self.zone();
        zone == TabBarZone::Left || zone == TabBarZone::Right
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TabEntry {
    pub item: TabBarItem,
    pub title: Line,
    x: usize,
    width: usize,
}

impl TabEntry {
    pub fn width(&self) -> usize {
        self.width
    }
}
