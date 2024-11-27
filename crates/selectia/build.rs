const TMP_DB_REL_PATH: &str = "../../target/selectia.db";

fn main() {
    regenerate_db();
    
    println!("cargo:rustc-env=DATABASE_URL={}", "sqlite://target/selectia.db");
}

fn regenerate_db() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let _ = tokio::fs::remove_file(&TMP_DB_REL_PATH).await;
        tokio::fs::File::create(&TMP_DB_REL_PATH).await.expect("Failed to create database file");

        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect(&format!("sqlite:{}", TMP_DB_REL_PATH))
            .await.unwrap();

        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    });
}
