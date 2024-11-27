#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    selectia_app_lib::run().await
}
