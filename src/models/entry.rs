use schema::entries;

#[derive(Debug, Identifiable, Queryable, Serialize)]
#[table_name="entries"]
pub struct Entry {
    pub id:   i64,
    pub hash: String,
    pub mime: Option<String>,
    pub is_orphan: Option<bool>,
}

#[derive(Insertable)]
#[table_name="entries"]
pub struct NewEntry<'a> {
    pub hash: &'a str,
    pub mime: Option<&'a str>,
}
