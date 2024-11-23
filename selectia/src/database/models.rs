use crate::prelude::*;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Metadata {
    pub id: i64,
    pub hash: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct File {
    pub id: i64,
    pub path: String,
    pub metadata_id: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MetadataTag {
    pub metadata_id: i64,
    pub tag_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tag {
    pub id: i64,
    pub name_id: i64,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TagName {
    pub id: i64,
    pub name: String,
    pub use_for_filtering: bool,
    pub index_in_ui: i64,
}

impl TagName {
    pub const FILE_NAME_EMBEDDING_ID: i64 = 1;
    pub const DIRECTORY_ID: i64 = 2;
    pub const FILE_NAME_ID: i64 = 3;
    pub const TITLE_ID: i64 = 4;
    pub const ARTIST_ID: i64 = 5;
    pub const ALBUM_ID: i64 = 6;
    pub const GENRE_ID: i64 = 7;
}
