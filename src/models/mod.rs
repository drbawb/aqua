#[derive(Debug, Queryable)]
pub struct Entry {
    pub id:   i64,
    pub hash: String,
    pub mime: Option<String>,
}

pub mod queries {
    use conduit::Request; // TODO: get yo server outta here
    use diesel::prelude::*;
    
    use super::Entry;
    use util::db::DbPool;

    pub fn all_entries(req: &Request) -> Vec<Entry> {
        use schema::entries::dsl::*;

        let conn = req.extensions().find::<DbPool>()
            .expect("could not load DB pooling extension")
            .get()
            .expect("could not fetch DB connection from pool");

        let results = entries.limit(5).load::<Entry>(&*conn);
        return results.unwrap();
    }
}
