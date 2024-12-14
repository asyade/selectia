use crate::{prelude::*, App};
use dto::{EntryChangedEvent, TagListChangedEvent};
use selectia::{analyser::entries_analyser::EntriesAnalyser, database};
use tauri::AppHandle;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct InteractiveListContext {
    handle: AppHandle,
    database: Database,
    cache: Arc<RwLock<Cache>>,
}

struct Cache {
    pub entries: Option<Vec<EntryView>>,
    pub filter: Option<EntryViewFilter>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            entries: None,
            filter: None,
        }
    }

    pub fn set(&mut self, entries: Vec<EntryView>, filter: EntryViewFilter) {
        self.entries = Some(entries);
        self.filter = Some(filter);
    }

    pub fn invalidate(&mut self, _database: &Database) {
        self.entries = None;
    }

    pub fn cached_entries(&self) -> Option<&Vec<EntryView>> {
        self.entries.as_ref()
    }

    pub async fn fill(&mut self, database: &Database) -> eyre::Result<()> {
        if let Some(filter) = &self.filter {
            self.entries = Some(database.get_entries(filter).await?);
            Ok(())
        } else {
            Err(eyre::eyre!("No filter in cache"))
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TagCreationResult {
    pub updated_entry: EntryView,
}

impl Context for InteractiveListContext {}

impl InteractiveListContext {
    pub async fn get_entries(&self, filter: EntryViewFilter) -> eyre::Result<Vec<EntryView>> {
        let entries = self.database.get_entries(&filter).await?;
        let mut lock = self.cache.write().await;
        lock.set(entries.clone(), filter);
        Ok(entries)
    }

    pub async fn get_tag_creation_suggestions(
        &self,
        tag_name_id: i64,
        input: String,
    ) -> eyre::Result<Vec<String>> {
        {
            info!(
                tag_name_id,
                input, "Getting tag creation suggestions (cached)"
            );
            let lock = self.cache.read().await;
            if let Some(entries) = lock.cached_entries() {
                return Ok(EntriesAnalyser::new(&entries)
                    .get_tag_creation_suggestions(tag_name_id, &input)?);
            }
        }

        {
            let mut lock = self.cache.write().await;
            lock.fill(&self.database).await?;
        }

        info!(
            tag_name_id,
            input, "Getting tag creation suggestions (uncached)"
        );
        let lock = self.cache.read().await;
        let entries = lock.cached_entries().unwrap();
        Ok(EntriesAnalyser::new(&entries).get_tag_creation_suggestions(tag_name_id, &input)?)
    }

    pub async fn create_tag(
        &self,
        metadata_id: i64,
        name_id: i64,
        value: String,
    ) -> eyre::Result<()> {
        info!(metadata_id, name_id, value, "Creating tag");
        self.database
            .set_metadata_tag_by_tag_name_id(metadata_id, name_id, value)
            .await?;
        let entry = self.database.get_entry_by_metadata_id(metadata_id).await?;
        {
            let mut lock = self.cache.write().await;
            lock.invalidate(&self.database);
        }

        self.handle.emit_event(EntryChangedEvent {
            entry: entry.into(),
        })?;
        self.handle.emit_event(TagListChangedEvent {})?;
        Ok(())
    }
}

impl InteractiveListContext {
    pub async fn new(app: &App) -> Self {
        let database = app
            .context
            .get_singleton::<Database>()
            .await
            .expect("Database singleton");
        let handle = app.context.get_singleton::<AppHandle>().await.expect("AppHandle singleton");
        Self {
            handle,
            database,
            cache: Arc::new(RwLock::new(Cache::new())),
        }
    }
}
