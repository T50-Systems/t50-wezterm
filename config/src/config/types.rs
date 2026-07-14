pub use wezterm_config_types::config::*;

use std::path::PathBuf;

pub(super) struct PathPossibility {
    pub(super) path: PathBuf,
    pub(super) is_required: bool,
}

impl PathPossibility {
    pub fn required(path: PathBuf) -> PathPossibility {
        PathPossibility {
            path,
            is_required: true,
        }
    }

    pub fn optional(path: PathBuf) -> PathPossibility {
        PathPossibility {
            path,
            is_required: false,
        }
    }
}
