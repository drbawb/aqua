#[macro_use] extern crate log;

extern crate aqua;
extern crate diesel;
extern crate dotenv;
extern crate env_logger;
extern crate rusqlite;

use aqua::models::{self, Entry, NewEntry, NewEntryTag, NewTag};
use aqua::schema;
use diesel::Connection as DieselConnection;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use std::collections::HashMap;
use std::env;
use rusqlite::Connection;

static CLIENT_DB_NAME:  &'static str = "C:\\Hydrus Network\\db\\client.db";
static MAPPING_DB_NAME: &'static str = "C:\\Hydrus Network\\db\\client.mappings.db";
static MASTER_DB_NAME:  &'static str = "C:\\Hydrus Network\\db\\client.master.db";

#[derive(Clone, Debug)]
struct ClientHash {
    pub hash_id: i64,
    pub md5:     Vec<u8>,
    pub sha1:    Vec<u8>,
    pub sha256:  Option<Vec<u8>>,
    pub sha512:  Vec<u8>,
}

#[derive(Clone, Debug)]
struct Mapping {
    pub ns_id:   i64,
    pub tag_id:  i64,
    pub hash_id: i64,
}

#[derive(Clone, Debug)]
struct Tag {
    pub tag_id: i64,
    pub ns_id:  i64,
    pub schema: String,
    pub name: String,
}

pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL not set");

    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

fn main() {
    dotenv::dotenv().expect("must provide .env file, see README (TODO: haha jk)");
    env_logger::init().expect("could not initialize console logging");
    
    let client_db  = Connection::open(CLIENT_DB_NAME).unwrap();
    let mapping_db = Connection::open(MAPPING_DB_NAME).unwrap();
    let master_db  = Connection::open(MASTER_DB_NAME).unwrap();

    info!("connected to hydrus database");

    let mut select_mappings = mapping_db
        .prepare("SELECT namespace_id, tag_id, hash_id FROM current_mappings_4")
        .unwrap();

    let mappings = select_mappings.query_map(&[], |row| {
        Mapping {
            ns_id:   row.get(0),
            tag_id:  row.get(1),
            hash_id: row.get(2),
        }
    });

    let mappings = mappings.unwrap()
      .map(|mb_row| mb_row.unwrap())
      .collect::<Vec<Mapping>>();

    info!("got {} mappings", mappings.len());

    let mut select_tags = master_db
        .prepare("SELECT tag_id, tag FROM tags WHERE tag_id = ?")
        .unwrap();

    let mut select_ns = master_db
        .prepare("SELECT namespace_id, namespace FROM namespaces WHERE namespace_id = ?")
        .unwrap();

    let mut tags = HashMap::new();
    for mapping in &mappings {
        let ns: String = select_ns.query_row(&[&mapping.ns_id], |row| {
            row.get(1)
        }).unwrap();

        let tag = select_tags.query_row(&[&mapping.tag_id], |row| {
            Tag { tag_id: row.get(0), ns_id: mapping.ns_id, schema: ns.clone(), name: row.get(1) }
        }).unwrap();

        tags.insert((tag.tag_id, ns.clone()), tag);
    }

    info!("got {} tags", tags.len());
    // let mut tags = vec![];


    // load the client hashes
    let mut select_hashes = client_db
        .prepare("SELECT hash_id, md5, sha1, sha512 FROM local_hashes")
        .unwrap();

    let hashes = select_hashes.query_map(&[], |row| {
        ClientHash {
            hash_id: row.get(0),
            md5:     row.get(1),
            sha1:    row.get(2),
            sha512:  row.get(3),
            sha256:  None, // filled in by master db
        }
    });

    let hash_records = hashes
        .unwrap()
        .map(|mb_row| mb_row.unwrap())
        .collect::<Vec<ClientHash>>();

    let mut hashes = HashMap::new();
    for record in &hash_records {
        hashes.insert(record.hash_id, record.clone());        
    }

    info!("loaded {} client files", hashes.len());

    // load remote hashes
    let mut select_hash = master_db
        .prepare("SELECT hash_id, hash FROM hashes WHERE hash_id = ?")
        .unwrap();

    for record in &hash_records {
        use std::collections::hash_map::Entry;

        let rhash = select_hash.query_row(&[&record.hash_id], |row| { 
            row.get(1) 
        }).unwrap();

        if let Entry::Occupied(mut o) = hashes.entry(record.hash_id) {
            o.get_mut().sha256 = Some(rhash);
        } else { panic!("unknown hash"); }
    }

    info!("loaded {} hashes", hashes.len());

    // begin population of our database
    let pg_conn = establish_connection();

    // map hydrus IDs => our IDs for later mapping restoration
    let mut aqua_entry_ids = HashMap::new(); 
    let mut aqua_tag_ids = HashMap::new();

    // load the entries
    for (hash_id, hash) in &hashes {
        let hash: Vec<String> = hash.sha256
            .as_ref().unwrap()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect();

        let hash: String = hash.join("");

        let entry = NewEntry { hash: &hash, mime: None };
        let entry: Entry = diesel::insert(&entry)
            .into(schema::entries::table)
            .get_result(&pg_conn).ok().unwrap();
        aqua_entry_ids.insert(hash_id, entry.id);
    }

    // load tags
    for (tag_id, otag) in &tags {
        // info!("tag({:?}) => {:?}", tag_id, tag);
        let new_tag = NewTag { name: &otag.name, schema: Some(&otag.schema) };
        let tag: models::Tag = diesel::insert(&new_tag)
            .into(schema::tags::table)
            .get_result(&pg_conn).ok().unwrap();
        aqua_tag_ids.insert((otag.ns_id, otag.tag_id), tag.id);
    }
    
    // load mappings
    for mapping in &mappings {
        let nentry = aqua_entry_ids.get(&mapping.hash_id).unwrap();
        let ntag   = aqua_tag_ids.get(&(mapping.ns_id, mapping.tag_id)).unwrap();
        let link   = NewEntryTag { tag_id: *ntag, entry_id: *nentry };

        let entry: models::EntryTag = diesel::insert(&link)
            .into(schema::entries_tags::table)
            .get_result(&pg_conn).ok().unwrap();
    }
}
