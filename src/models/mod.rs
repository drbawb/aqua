use schema::{entries_tags, entries, tags};

#[derive(Debug, Queryable)]
pub struct Entry {
    pub id:   i64,
    pub hash: String,
    pub mime: Option<String>,
}

#[derive(Insertable)]
#[table_name="entries"]
pub struct NewEntry<'a> {
    pub hash: &'a str,
    pub mime: Option<&'a str>,
}

#[derive(Debug, Queryable)]
pub struct Tag {
    pub id:     i64,
    pub schema: Option<String>,
    pub name:   String,
}

#[derive(Insertable)]
#[table_name="tags"]
pub struct NewTag<'a> {
    pub schema: Option<&'a str>,
    pub name: &'a str,
}

#[derive(Debug, Queryable, Serialize)]
pub struct EntryTag {
    pub tag_id:   i64,
    pub entry_id: i64,
}

#[derive(Insertable)]
#[table_name="entries_tags"]
pub struct NewEntryTag {
    pub tag_id:   i64,
    pub entry_id: i64,
}

pub mod queries {
    use aqua_web::plug;
    use diesel;
    use diesel::prelude::*;
    
    use models::{Entry, EntryTag, NewEntry, Tag};
    use util::db::require_db_conn;

    pub fn find_entry(conn: &plug::Conn, entry_id: i64) -> Option<Entry> {
        use schema::entries::dsl::*;

        let conn = require_db_conn(conn);
        entries.filter(id.eq(entry_id))
            .get_result(&*conn)
            .ok()
    }

    // TODO: join these through many<->many
    pub fn find_entries_for(conn: &plug::Conn, dest_tag_id: i64) -> Option<Vec<EntryTag>> {
        use schema::entries_tags::dsl::*;

        let conn = require_db_conn(conn);
        entries_tags.filter(tag_id.eq(dest_tag_id))
            .load(&*conn)
            .ok()
    }

    pub fn all_entries(conn: &plug::Conn) -> Vec<Entry> {
        use schema::entries::dsl::*;

        let conn = require_db_conn(conn);
        let results = entries.limit(5).load::<Entry>(&*conn);
        return results.unwrap();
    }

    pub fn find_or_insert<'a>(conn: &mut plug::Conn, entry: NewEntry<'a>) -> Option<Entry> {
        use schema::entries::dsl::*;

        let conn = require_db_conn(conn);
        diesel::insert(&entry)
            .into(entries)
            .get_result(&*conn).ok()
    }


    pub fn find_tag(conn: &plug::Conn, schema_name: &str, tag_name: &str) -> Option<Tag> {
        use schema::tags::dsl::*;

        let conn = require_db_conn(conn);
        tags.filter(name.eq(tag_name))
            .filter(schema.eq(schema_name))
            .get_result(&*conn)
            .ok()
    } 
}
