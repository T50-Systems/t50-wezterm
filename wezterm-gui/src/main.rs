// Don't create a new standard console window when launched from the windows GUI.
#![cfg_attr(not(test), windows_subsystem = "windows")]

mod colorease;
mod commands;
mod customglyph;
mod download;
mod frontend;
mod glyphcache;
mod inputmap;
mod overlay;
mod quad;
mod renderstate;
mod resize_increment_calculator;
mod scripting;
mod scrollbar;
mod selection;
mod shapecache;
mod spawn;
mod stats;
mod tabbar;
mod termwindow;
mod unicode_names;
mod uniforms;
mod update;
mod utilsprites;

include!("main/cli.rs");
include!("main/remote.rs");
include!("main/startup.rs");
include!("main/publish.rs");
include!("main/errors.rs");
include!("main/ls_fonts.rs");
include!("main/run.rs");
