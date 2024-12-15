use std::ops::Deref;

use crate::prelude::*;
use tempdir::TempDir;

pub struct TmpDatabase {
    dir: TempDir,
    database: Database,
}

impl TmpDatabase {
    pub async fn new() -> Self {
        let dir = TempDir::new("selectia").unwrap();
        let database_path = dir.path().join("database.db");
        let database = Database::new(&database_path).await.unwrap();
        Self { dir, database }
    }
}

impl Deref for TmpDatabase {
    type Target = Database;

    fn deref(&self) -> &Self::Target {
        &self.database
    }
}
