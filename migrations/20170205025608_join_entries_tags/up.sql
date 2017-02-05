CREATE TABLE entries_tags (
    tag_id   bigint REFERENCES tags (id),
    entry_id bigint REFERENCES entries (id),

    PRIMARY KEY (tag_id, entry_id)
);

CREATE INDEX entries_tags_tag_id_idx ON entries_tags (tag_id);
CREATE INDEX entries_tags_entry_id_idx ON entries_tags (entry_id);
