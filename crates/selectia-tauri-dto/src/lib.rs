use std::path::Path;

use ts_rs::TS;

pub mod prelude;
pub mod events;
pub mod models;

pub fn manual_export(to_directory: &Path) {
    std::env::set_var("TS_RS_EXPORT_DIR", to_directory);
    models::Models::export_all().expect("Failed to export models");
    events::Events::export_all().expect("Failed to export events");
}
