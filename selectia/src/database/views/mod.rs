use crate::prelude::*;
pub mod entry_view;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagView {
    pub id: i64,
    pub name_id: i64,
    pub value: String,
}

impl From<Tag> for TagView {
    fn from(tag: Tag) -> Self {
        Self { id: tag.id, name_id: tag.name_id, value: tag.value }
    }
}
