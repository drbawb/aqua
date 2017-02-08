use schema::entries;

#[derive(Debug, Identifiable, Queryable, Serialize)]
#[table_name="entries"]
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
