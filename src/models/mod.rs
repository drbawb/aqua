use iron::request::Request;
use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;

use util::db::DbPool;

pub type ConcretePool = Pool<PostgresConnectionManager>;

fn fetch_db_context(req: &Request) -> ConcretePool {
    req.extensions.get::<DbPool>()
                  .expect("DB context not found in extensions")
                  .clone()
}

#[derive(Debug)]
pub struct Entry {
    id:   i64,
    hash: String,
    mime: String,
}

impl Entry {
    pub fn find_by_hash(req: &Request, hash: &str) -> Option<Entry> {
        let db_pool = fetch_db_context(req);
        let conn = db_pool.get().expect("db pool did not produce connection");

        // find matching record
        let query = "SELECT id,hash,mime FROM entries WHERE hash = $1";
        for row in &conn.query(query, &[&hash]).unwrap() {
            let entry = Entry {
                id:   row.get::<_,i32>(0) as i64,
                hash: row.get(1),
                mime: row.get(2),
            };

            return Some(entry);
        }

        None
    }
}
