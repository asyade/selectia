fn main() {
    dotenvy::from_filename_override("build.env").unwrap();
    println!("cargo:rustc-env=TS_RS_EXPORT_DIR={}", std::env::var("TS_RS_EXPORT_DIR").unwrap());

    tauri_build::build()
}
