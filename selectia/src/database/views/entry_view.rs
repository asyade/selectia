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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryViewFilter {
    directories: Vec<String>,
    tags: HashMap<i64, Vec<TagFilter>>,
    offset: Option<i64>,
    limit: Option<i64>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagFilter {
    id: i64,
    selected: bool,
}

impl EntryViewFilter {
    const QUERY_HEAD: &str = r#"
        SELECT metadata.id as metadata_id, metadata.hash as metadata_hash, json_group_array(json_object(
            'tag_id', tagged_metadata.tag_id, 
            'metadata_tag_id', tagged_metadata.metadata_id,
            'tag_name_id', tagged_metadata.tag_name_id, 
            'tag_value', tagged_metadata.tag_value
        )) as tags FROM metadata
            LEFT JOIN tagged_metadata on tagged_metadata.metadata_id = metadata.id
        WHERE metadata.id IN (
            SELECT metadata_id FROM tagged_metadata
    "#;

    const QUERY_FOOT: &str = r#"
            GROUP BY metadata_id
        )
        GROUP BY metadata.id
    "#;

    pub async fn query(&self, pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<Vec<EntryView>> {
        let mut builder = QueryBuilder::<sqlx::Sqlite>::new(Self::QUERY_HEAD);

        for (idx, (name_id, tag)) in self.tags.iter().flat_map(|(named_id, e)| e.iter().map(|tag| (*named_id, tag))).enumerate() {
            if idx == 0 {
                builder.push("WHERE ");
            } else {
                builder.push("AND ");
            }
            builder.push("tag_name_id = ");
            builder.push_bind(name_id);
            builder.push(" AND tag_id = ");
            builder.push_bind(tag.id);
            builder.push("\n");
        }

        builder.push(Self::QUERY_FOOT);
        let query: String = builder.build().sql().into();
        info!("Generated query: {}", &query);
        let entries = sqlx::query_as(query.as_str()).fetch_all(pool).await?;
        Ok(entries)
    }
}
