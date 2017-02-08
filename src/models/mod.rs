mod entry;
mod entry_tag;
mod tag;

pub use self::entry::{Entry, NewEntry};
pub use self::entry_tag::{EntryTag, NewEntryTag};
pub use self::tag::{Tag, NewTag};

pub mod queries {
    use aqua_web::plug;
    use diesel;
    use diesel::prelude::*;

    use models::entry::{Entry, NewEntry};
    use models::entry_tag::EntryTag;
    use models::tag::Tag;

    use util::db;

    pub fn find_entry(conn: &plug::Conn, entry_id: i64) -> db::Result<Entry> {

        use schema::entries::dsl::*;
        let conn = db::fetch_conn(conn)?;
        let entry = entries.filter(id.eq(entry_id))
            .get_result(&*conn)?;

        Ok(entry)
    }

    pub fn find_entry_by_hash(conn: &plug::Conn, entry_hash: &str) -> db::Result<Option<Entry>> {
        use schema::entries::dsl::*;
        let conn = db::fetch_conn(conn)?;
        let entry = entries.filter(hash.eq(entry_hash))
            .get_result(&*conn)
            .optional()?;

        Ok(entry)
    }

    // TODO: join these through many<->many
    pub fn find_entries_for(conn: &plug::Conn, dest_tag_id: i64) -> db::Result<Vec<EntryTag>> {
        use schema::entries_tags::dsl::*;

        let conn = db::fetch_conn(conn)?;
        let results = entries_tags
            .filter(tag_id.eq(dest_tag_id))
            .load(&*conn)?;

        Ok(results)
    }


    // TODO: join these through many <-> many
    pub fn find_tags_for(conn: &plug::Conn, dest_entry_id: i64) -> db::Result<Vec<Tag>> {
        use schema::{entries_tags, tags};

        let conn = db::fetch_conn(conn)?;
        let results = entries_tags::table
            .inner_join(tags::table)
            .filter(entries_tags::entry_id.eq(dest_entry_id))
            .load(&*conn)?.into_iter()
            .map(|(_assoc, tag): (EntryTag, Tag)| { tag })
            .collect();

        Ok(results)   
    }

    pub fn all_entries(conn: &plug::Conn) -> Vec<Entry> {
        use schema::entries::dsl::*;

        let conn = db::fetch_conn(conn).unwrap();
        let results = entries.limit(5).load::<Entry>(&*conn);
        return results.unwrap();
    }

    pub fn find_or_insert<'a>(conn: &mut plug::Conn, entry: NewEntry<'a>) -> Option<Entry> {
        use schema::entries::dsl::*;
        let conn = db::fetch_conn(conn).unwrap();
        diesel::insert(&entry)
            .into(entries)
            .get_result(&*conn).ok()
    }


    pub fn find_tag(conn: &plug::Conn, schema_name: &str, tag_name: &str) -> db::Result<Tag> {
        use schema::tags::dsl::*;

        let conn = db::fetch_conn(conn)?;
        let tag = tags.filter(name.eq(tag_name))
            .filter(schema.eq(schema_name))
            .get_result(&*conn)?;

        Ok(tag)
    } 
}
