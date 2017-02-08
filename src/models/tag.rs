use schema::{entries_tags, tags};

#[derive(Debug, Associations, Identifiable, Queryable, Serialize)]
#[table_name="tags"]
#[has_many(entries_tags)]
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
