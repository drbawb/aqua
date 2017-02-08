ALTER TABLE entries_tags
ADD COLUMN id BIGSERIAL,
DROP CONSTRAINT entries_tags_pkey,
ADD CONSTRAINT entries_tags_pkey PRIMARY KEY (id),
ADD CONSTRAINT entries_tags_entry_id_tag_id UNIQUE (entry_id, tag_id)
