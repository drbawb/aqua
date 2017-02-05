CREATE TABLE tags (
    id bigserial PRIMARY KEY,
    schema  character varying,
    name    character varying NOT NULL,
    
    CONSTRAINT tags_schema_name UNIQUE (schema,name)
)
