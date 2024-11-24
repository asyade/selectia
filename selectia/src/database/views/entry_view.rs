use crate::prelude::*;
use sqlx::{prelude::*, Execute, QueryBuilder};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EntryView {
    pub metadata_id: i32,
    pub metadata_hash: String,
    pub tags: sqlx::types::Json<Vec<MetadataTagView>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataTagView {
    pub tag_id: i64,
    pub metadata_tag_id: i64,
    pub tag_name_id: i64,
    pub tag_value: String,
    pub metadata_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryViewFilter {
    directories: Vec<String>,
    tags: HashMap<i64, Vec<TagFilter>>,
    offset: Option<i64>,
    limit: Option<i64>,
}

impl PartialEq for EntryViewFilter {
    fn eq(&self, other: &Self) -> bool {
        if self.directories == other.directories && self.offset == other.offset && self.limit == other.limit {
            for (key, value) in &self.tags {
                if other.tags.get(key).map(|v| v != value).unwrap_or(true) {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}


impl EntryView {
        const QUERY_HEAD: &str = r#"
        SELECT metadata.id as metadata_id, metadata.hash as metadata_hash, json_group_array(json_object(
            'tag_id', tagged_metadata.tag_id, 
            'metadata_tag_id', tagged_metadata.metadata_id,
            'tag_name_id', tagged_metadata.tag_name_id, 
            'tag_value', tagged_metadata.tag_value,
            'metadata_id', metadata.id
        )) as tags FROM metadata
            LEFT JOIN tagged_metadata on tagged_metadata.metadata_id = metadata.id
        WHERE metadata.id IN (
            SELECT DISTINCT metadata_id FROM tagged_metadata as tm
    "#;

    const QUERY_FOOT: &str = r#"
        )
        GROUP BY metadata.id
    "#;

    pub async fn get_one_by_metadata_id(metadata_id: i64, pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<EntryView> {
        let mut builder = QueryBuilder::<sqlx::Sqlite>::new(EntryView::QUERY_HEAD);

        builder.push("WHERE metadata.id = ");
        builder.push_bind(metadata_id);

        builder.push(EntryView::QUERY_FOOT);
        let query: String = builder.build().sql().into();
        let entry = sqlx::query_as(query.as_str()).bind(metadata_id).fetch_one(pool).await?;
        Ok(entry)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TagFilter {
    id: i64,
    selected: bool,
}

impl EntryViewFilter {

    pub async fn query(&self, pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<Vec<EntryView>> {
        let mut builder = QueryBuilder::<sqlx::Sqlite>::new(EntryView::QUERY_HEAD);

        let tags = self.tags.iter().flat_map(|(named_id, e)| e.iter().map(|tag| (*named_id, tag))).collect::<Vec<_>>();
        for (idx, (name_id, tag)) in tags.iter().enumerate() {
            if idx == 0 {
                if tags.len() > 1 {
                    builder.push("WHERE ((");
                } else {
                    builder.push("WHERE (");
                }
            } else {
                builder.push(" OR (");
            }
            builder.push("tm.tag_name_id = ");
            builder.push_bind(name_id);
            builder.push(" AND tm.tag_id = ");
            builder.push_bind(tag.id);
            builder.push(")");

            if idx == tags.len() - 1 && tags.len() > 1 {
                builder.push(")");
            }
        }

        builder.push(EntryView::QUERY_FOOT);
        let query: String = builder.build().sql().into();
        let mut query = sqlx::query_as(query.as_str());
        for (name_id, tag) in self.tags.iter().flat_map(|(named_id, e)| e.iter().map(|tag| (*named_id, tag))) {
            query = query.bind(name_id);
            query = query.bind(tag.id);
        }
        info!("Generated query: {}", query.sql().to_string());
        let entries = query.fetch_all(pool).await?;
        Ok(entries)
    }
}
