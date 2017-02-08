use models::tag::Tag;
use schema::entries_tags;

#[derive(Debug, Associations, Identifiable, Queryable, Serialize)]
#[table_name="entries_tags"]
#[belongs_to(Tag)]
pub struct EntryTag {
    pub id:       i64,
    pub tag_id:   i64,
    pub entry_id: i64,
}

#[derive(Insertable)]
#[table_name="entries_tags"]
pub struct NewEntryTag {
    pub tag_id:   i64,
    pub entry_id: i64,
}
