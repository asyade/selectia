CREATE TABLE tag_name (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name TEXT NOT NULL,
    use_for_filtering BOOLEAN NOT NULL DEFAULT TRUE,
    index_in_ui INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE tag (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name_id INTEGER NOT NULL,
    value TEXT NOT NULL,
    FOREIGN KEY (name_id) REFERENCES tag_name(id)
);

CREATE TABLE file (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    path TEXT NOT NULL UNIQUE,
    metadata_id INTEGER NOT NULL,
    FOREIGN KEY (metadata_id) REFERENCES metadata(id)
);

CREATE TABLE metadata (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    hash TEXT NOT NULL
);

CREATE TABLE metadata_tag (
    metadata_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    FOREIGN KEY (metadata_id) REFERENCES metadata(id),
    FOREIGN KEY (tag_id) REFERENCES tag(id)
);

CREATE VIEW tagged_metadata AS
SELECT
    metadata_tag.metadata_id as metadata_id,
    tag.id as tag_id,
    tag.name_id as tag_name_id,
    tag.value as tag_value
FROM metadata_tag
    LEFT JOIN tag on tag.id = metadata_tag.tag_id;
