use selectia::analyser::EntriesAnalyser;
use tokio::sync::RwLock;
use crate::{prelude::*, App};

#[derive(Clone)]
pub struct InteractiveListContext {
    pub app: App,
    /// Cache to limit database queries
    /// TODO: Use a more efficient cache and store multiple queries results ?
    pub cache: Arc<RwLock<Option<(Vec<EntryView>, EntryViewFilter)>>>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TagCreationResult {
    pub updated_entry: EntryView,
}

impl Context for InteractiveListContext {}

impl InteractiveListContext {
    pub async fn get_entries(&self, filter: EntryViewFilter) -> eyre::Result<Vec<EntryView>> {
        let entries = self.app.get_entries(&filter).await?;
        let mut lock = self.cache.write().await;
        *lock = Some((entries.clone(), filter));
        Ok(entries)
    }

    pub async fn get_tag_creation_suggestions(&self, tag_name_id: i64, input: String) -> eyre::Result<Vec<String>> {
        info!(tag_name_id, input, "Getting tag creation suggestions");
        let lock = self.cache.read().await;
        let (entries, _) = lock.as_ref().unwrap();
        let analyser = EntriesAnalyser::new(&entries);
        analyser.get_tag_creation_suggestions(tag_name_id, &input)
    }

    pub async fn create_tag(&self, metadata_id: i64, name_id: i64, value: String) -> eyre::Result<EntryView> {
        info!(metadata_id, name_id, value, "Creating tag");
        self.app.database.set_metadata_tag_by_tag_name_id(metadata_id, name_id, value).await?;
        let entry = self.app.database.get_entry_by_metadata_id(metadata_id).await?;
        self.invalidate_cache().await;
        Ok(entry.into())
    }

    async fn invalidate_cache(&self) {
        let mut lock = self.cache.write().await;
        *lock = None;
    }
}

impl InteractiveListContext {
    pub fn new(app: App) -> Self {
        Self { app, cache: Arc::new(RwLock::new(None)) }
    }
}
