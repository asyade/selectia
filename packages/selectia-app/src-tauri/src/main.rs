#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing_subscriber::fmt::format::PrettyFields;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .pretty()
        .with_file(true)
        .fmt_fields(PrettyFields::new())
        .init();

    selectia_app_lib::run().await
}
