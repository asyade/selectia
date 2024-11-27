use crate::prelude::*;
use crate::views::entry_view::*;
pub mod models;
pub mod views;

#[derive(Clone)]
pub struct Database {
    pool: sqlx::Pool<sqlx::Sqlite>,
}

impl Database {
    #[instrument]
    pub async fn new(path: &Path) -> Result<Self> {
        if !path.exists() {
            info!("Creating database");
            std::fs::File::create(path).unwrap();
        } else {
            info!("Database already exists");
        }

        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect(&format!("sqlite:{}", path.to_str().unwrap()))
            .await?;

        info!("Running DB migrations");
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Error running DB migrations");

        info!("Database service ready !");
        Ok(Self { pool })
    }

    pub async fn get_or_create_metadata(&self, hash: &str) -> Result<(models::Metadata, bool)> {
        let metadata = sqlx::query_as!(
            models::Metadata,
            "SELECT * FROM metadata WHERE hash = ?",
            hash
        )
        .fetch_optional(&self.pool)
        .await?;
        match metadata {
            Some(metadata) => Ok((metadata, false)),
            None => {
                info!("Creating metadata for {}", hash);
                let metadata = sqlx::query_as!(
                    models::Metadata,
                    "INSERT INTO metadata (hash) VALUES (?) RETURNING *",
                    hash
                )
                .fetch_one(&self.pool)
                .await?;
                Ok((metadata, true))
            }
        }
    }

    pub async fn create_or_replace_file(
        &self,
        path: &Path,
        metadata_id: i64,
    ) -> Result<models::File> {
        let path_str = path.to_str().unwrap();
        let file = sqlx::query_as!(models::File, "INSERT INTO file (path, metadata_id) VALUES (?, ?) ON CONFLICT(path) DO UPDATE SET metadata_id = ? RETURNING *", path_str, metadata_id, metadata_id).fetch_one(&self.pool).await?;
        Ok(file)
    }

    pub async fn list_files(&self) -> Result<Vec<models::File>> {
        let files = sqlx::query_as!(models::File, "SELECT * FROM file")
            .fetch_all(&self.pool)
            .await?;
        Ok(files)
    }

    pub async fn set_metadata_tag_by_tag_name_id(
        &self,
        metadata_id: i64,
        tag_name_id: i64,
        value: String,
    ) -> Result<()> {
        let existing_tag = match sqlx::query_scalar!(
            "SELECT id FROM tag WHERE name_id = ? AND value = ?",
            tag_name_id,
            value
        )
        .fetch_optional(&self.pool)
        .await?
        {
            Some(tag_id) => tag_id,
            None => {
                sqlx::query_scalar!(
                    "INSERT INTO tag (name_id, value) VALUES (?, ?) RETURNING id",
                    tag_name_id,
                    value
                )
                .fetch_one(&self.pool)
                .await?
            }
        };
        self.set_metadata_tag(metadata_id, existing_tag).await?;
        Ok(())
    }

    pub async fn set_tag(&self, name: &str, value: String) -> Result<i64> {
        let tag_name_id = sqlx::query_scalar!("SELECT id FROM tag_name WHERE name = ?", name)
            .fetch_one(&self.pool)
            .await?;
        let tag_id = sqlx::query_scalar!(
            "INSERT INTO tag (name_id, value) VALUES (?, ?) RETURNING id",
            tag_name_id,
            value
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(tag_id)
    }

    ///  TODO: reduce number of queries
    pub async fn set_metadata_tag(&self, metadata_id: i64, tag_id: i64) -> Result<()> {
        sqlx::query!(
            "INSERT INTO metadata_tag (metadata_id, tag_id) VALUES (?, ?)",
            metadata_id,
            tag_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_tags(&self, name: &str) -> Result<Vec<models::Tag>> {
        let tag_name_id = sqlx::query_scalar!("SELECT id FROM tag_name WHERE name = ?", name)
            .fetch_one(&self.pool)
            .await?;
        let tags = sqlx::query_as!(
            models::Tag,
            "SELECT * FROM tag WHERE name_id = ?",
            tag_name_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(tags)
    }

    pub async fn get_tag_names(&self) -> Result<Vec<models::TagName>> {
        let tag_names = sqlx::query_as!(
            models::TagName,
            "SELECT * FROM tag_name ORDER BY index_in_ui ASC"
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(tag_names)
    }

    pub async fn get_tags_by_name(&self, tag_name: &str) -> Result<Vec<models::Tag>> {
        let tag_name_id = sqlx::query_scalar!("SELECT id FROM tag_name WHERE name = ?", tag_name)
            .fetch_one(&self.pool)
            .await?;
        let tags = sqlx::query_as!(
            models::Tag,
            "SELECT * FROM tag WHERE name_id = ?",
            tag_name_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(tags)
    }

    pub async fn get_entries(&self, filter: &EntryViewFilter) -> Result<Vec<EntryView>> {
        info!("Getting entries");
        Ok(filter.query(&self.pool).await?)
    }

    pub async fn get_entry_by_metadata_id(&self, metadata_id: i64) -> Result<EntryView> {
        EntryView::get_one_by_metadata_id(metadata_id, &self.pool).await
    }

    pub async fn create_task(&self, payload: String) -> Result<i64> {
        let task = sqlx::query_scalar!(
            "INSERT INTO task (payload) VALUES (?) RETURNING id",
            payload
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(task)
    }

    pub async fn get_task(&self, id: i64) -> Result<models::Task> {
        let task = sqlx::query_as!(models::Task, "SELECT * FROM task WHERE id = ?", id)
            .fetch_one(&self.pool)
            .await?;
        Ok(task)
    }

    pub async fn get_tasks(&self) -> Result<Vec<models::Task>> {
        let tasks = sqlx::query_as!(models::Task, "SELECT * FROM task")
            .fetch_all(&self.pool)
            .await?;
        Ok(tasks)
    }

    pub async fn delete_task(&self, id: i64) -> Result<()> {
        sqlx::query!("DELETE FROM task WHERE id = ?", id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Dequeue a task from the queue
    /// This function will set the status of the task first task with queued status to processing
    ///
    /// Returns the task or none if no task is in queued status
    pub async fn dequeue_task(&self) -> Result<Option<models::Task>> {
        let task = sqlx::query_as!(models::Task, r#"
            UPDATE task SET status = 'processing'
            WHERE status = 'queued' AND id = (SELECT id FROM task WHERE status = 'queued' ORDER BY id ASC LIMIT 1)
            RETURNING *
        "#)
            .fetch_optional(&self.pool)
            .await?;
        Ok(task)
    }

    /// Sanitize task status to ensure that no task is in processing status
    /// This function should be called when before worker start processing tasks
    pub async fn sanitize_task_status(&self) -> Result<u64> {
        let result = sqlx::query!("UPDATE task SET status = 'queued' WHERE status = 'processing'")
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn get_file_from_metadata_id(&self, metadata_id: i64) -> Result<models::File> {
        let file = sqlx::query_as!(
            models::File,
            "SELECT * FROM file WHERE metadata_id = ?",
            metadata_id
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(file)
    }
}
