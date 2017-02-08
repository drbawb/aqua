ALTER TABLE entries_tags
DROP COLUMN id,
DROP CONSTRAINT entries_tags_entry_id_tag_id,
ADD CONSTRAINT entries_tags_pkey PRIMARY KEY (entry_id, tag_id)
