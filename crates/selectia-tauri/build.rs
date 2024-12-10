use std::{collections::{HashMap}, path::PathBuf};

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:info={}", format!($($tokens)*))
    }
}
pub fn main() {
    dotenv::from_filename("build.env").ok();
    let ts_mirror_package = PathBuf::from(std::env::var("TS_MIRROR_PACKAGE").unwrap());
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("generated_typescript");
    let _ =std::fs::remove_dir_all(&out_dir);
    std::fs::create_dir_all(&out_dir).unwrap();
    selectia_tauri_dto::manual_export(&out_dir);

    let generated_files = std::fs::read_dir(&out_dir).unwrap().filter_map(|e| {
        let path = e.ok()?.path();
        Some((path.file_name().unwrap().to_str().unwrap().to_string(), path))
    }).collect::<HashMap<_, _>>();
    let mut existing_files = std::fs::read_dir(&ts_mirror_package).unwrap().filter_map(|e| {
        let path = e.ok()?.path();
        Some((path.file_name().unwrap().to_str().unwrap().to_string(), path))
    }).collect::<HashMap<_, _>>();

    p!("{} generated files", generated_files.len());
    for file_name in generated_files.keys() {
        if let Some(existing_file) = existing_files.get(file_name) {
            if !file_eq(&generated_files[file_name], existing_file) {
                p!("File different, copy new version");
                std::fs::copy(&generated_files[file_name], &ts_mirror_package.join(file_name)).unwrap();
            } else {
                p!("File is the same, skip");
            }
        } else {
            p!("File does not exist in ts_mirror_package, copy new version");
            std::fs::copy(&generated_files[file_name], &ts_mirror_package.join(file_name)).unwrap();
        }
        existing_files.remove(file_name);
    }

    for file in existing_files.values() {
        p!("File does not exist in generated_typescript, remove");
        std::fs::remove_file(file).unwrap();
    }

}

fn file_eq(a: &PathBuf, b: &PathBuf) -> bool {
    std::fs::read_to_string(a).unwrap() == std::fs::read_to_string(b).unwrap()
}
