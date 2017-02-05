#[macro_use] extern crate log;

extern crate aqua;
extern crate diesel;
extern crate dotenv;
extern crate env_logger;
extern crate glob;

use std::env;
use std::fs;
use std::os;
use std::path::PathBuf;

use aqua::models::{Entry, EntryTag};
use aqua::schema;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use glob::glob;

static BASE_PATH: &'static str = "C:\\Hydrus Network\\db\\client_files";
static LINK_PATH: &'static str = "C:\\aqua_test_link";

fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL not set");

    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

fn find_entry(conn: &PgConnection, entry_id: i64) -> Option<Entry> {
    use schema::entries::dsl::*;

    entries.filter(id.eq(entry_id))
        .get_result(conn)
        .ok()
}

// TODO: join these through many<->many
fn find_entries_for(conn: &PgConnection, dest_tag_id: i64) -> Option<Vec<EntryTag>> {
    use schema::entries_tags::dsl::*;

    entries_tags.filter(tag_id.eq(dest_tag_id))
        .load(conn)
        .ok()
}

fn path_for(entry: Entry) -> PathBuf {
    let path_glob = format!("{}\\f{}\\{}.*",
                            BASE_PATH,
                            &entry.hash[0..2],
                            &entry.hash);

    println!("glob pattern: {}", path_glob);
    let mut paths = glob(&path_glob)
        .expect("could not parse glob pattern")
        .map(|res| res.ok().unwrap())
        .collect::<Vec<PathBuf>>();

    assert_eq!(paths.len(), 1);
    paths.remove(0)
}

fn main() {
    dotenv::dotenv().expect("must provide .env file, see README (TODO: haha jk)");
    env_logger::init().expect("could not initialize console logging");

    info!("hello, world...");
    info!("got tag id {}", env::args().nth(1).unwrap());

    let tag_id: i64 = env::args().nth(1)
        .expect("must provide a tag id")
        .parse()
        .expect("tag id must be a number");

    let db_conn = establish_connection();

    let entries = find_entries_for(&db_conn, tag_id)
        .unwrap_or(vec![])
        .into_iter()
        .map(|entry_tag| find_entry(&db_conn, entry_tag.entry_id).unwrap())
        .map(|entry| path_for(entry))
        .collect::<Vec<_>>();

    info!("creating links for {} entries", entries.len());
    for entry in &entries {
        let dst = PathBuf::from(LINK_PATH).join(entry.file_name().unwrap());
        println!("{:?} => {:?}", entry, dst);
        match fs::hard_link(entry, dst) {
            Err(msg) => info!("error linking: {}", msg),
            _ => {},
        }
        // let dst_path = format!("{}/{}", LINK_PATH, entry.file_name().unwrap());
        // fs::soft_link(entry, );        
    }


    // // load entry pointers for this tag
    // let results = queries::find_tag(conn, &schema_name, &tag_name)
    //     .and_then(|tag| queries::find_entries_for(conn, tag.id))
    //     .unwrap_or(vec![]);

    // let data = EntryListView { entries: results };
    // let view = views::render_into(conn.req(), "layouts/main", "dash/list", &data);
    // conn.send_resp(200, &view);

}
