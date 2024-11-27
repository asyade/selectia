use std::path::PathBuf;


pub fn main() {
    dotenv::from_filename("build.env").ok();

    let ts_mirror_package = std::env::var("TS_MIRROR_PACKAGE").unwrap();
    selectia_tauri_dto::manual_export(&PathBuf::from(ts_mirror_package));
}
