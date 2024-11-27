use crate::{prelude::*, AppResult};
use tokio::sync::RwLock;

pub mod interactive_list_context;

#[derive(Clone)]
pub struct ContextProvider<T: Context> {
    contextes: Arc<RwLock<HashMap<String, T>>>,
}

pub struct ContextId(uuid::Uuid);

impl<T: Context> ContextProvider<T> {
    pub fn new() -> Self {
        Self {
            contextes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_context(&self, context: T) -> ContextId {
        let id = ContextId(uuid::Uuid::new_v4());
        self.contextes.write().await.insert(id.to_string(), context);
        id
    }

    pub async fn get_context(&self, id: ContextId) -> eyre::Result<T> {
        self.contextes.read().await.get(&id.to_string()).cloned().ok_or_else(|| eyre::eyre!("Context not found"))
    }

    pub async fn delete_context(&self, id: ContextId) -> eyre::Result<()> {
        self.contextes.write().await.remove(&id.to_string()).ok_or_else(|| eyre::eyre!("Context not found"))?;
        Ok(())
    }
}

impl std::fmt::Display for ContextId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for ContextId {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(uuid::Uuid::parse_str(&value)?))
    }
}

pub trait Context: Send + Sync + Clone {
}

