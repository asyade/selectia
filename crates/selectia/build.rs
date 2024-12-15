use tempdir::TempDir;


fn main() {
    regenerate_db();
    
    println!("cargo:rustc-env=DATABASE_URL={}", "sqlite://target/selectia.db");
}

fn regenerate_db() {
    let tmpdir = TempDir::new("selectia-build").unwrap();
    let tmp_db_rel_path = tmpdir.path().join("selectia.db");

    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let _ = tokio::fs::remove_file(&tmp_db_rel_path).await;
        tokio::fs::File::create(&tmp_db_rel_path).await.expect("Failed to create database file");

        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect(&format!("sqlite:{}", tmp_db_rel_path.display()))
            .await.unwrap();

        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    });
}
