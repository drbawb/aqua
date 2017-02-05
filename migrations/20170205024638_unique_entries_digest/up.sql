ALTER TABLE entries 
ADD CONSTRAINT entries_hash_unique UNIQUE (hash);
