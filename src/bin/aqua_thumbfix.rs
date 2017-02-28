#[macro_use] extern crate log;

extern crate aqua;
extern crate clap;
extern crate diesel;
extern crate dotenv;
extern crate env_logger;

use aqua::models::{Entry, EntryTag, Tag};
use aqua::schema;
use aqua::util::processing;
use clap::{Arg, App};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;
use std::path::PathBuf;

fn main() {
    dotenv().expect("must provide .env file, see README (TODO: haha jk)");
    env_logger::init().expect("could not initialize console logging");

    // read command line arguments
    let matches = App::new("aqua-watch")
        .version("0.1.0")
        .author("himechi <hime@localhost>")
        .about("Watches a directory for new files and moves them to the `aqua` content store.")
        .arg(Arg::with_name("CONTENT_PATH")
             .help("Determines the input directory to be watched.")
             .required(true)
             .index(1))
        .get_matches();


    let content_store = matches.value_of("CONTENT_PATH").unwrap();

    match process_entries(&content_store[..]) {
        Ok(_) => info!("a-ok!"),
        Err(msg) => warn!("thumbfix encountered an error: {:?}", msg),
    }
}

fn establish_connection() -> processing::Result<PgConnection> {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL not set in `.env` file !!!");

    Ok(PgConnection::establish(&database_url)?)
}

fn process_entries(content_store: &str) -> processing::Result<()> {
    let conn = establish_connection()?;
    
    let missing_thumb_tag = schema::tags::table
        .filter(schema::tags::name.eq("THUMB"))
        .get_result::<Tag>(&conn)?;

    let entries = schema::entries::table
        .inner_join(schema::entries_tags::table)
        .filter(schema::entries_tags::tag_id.eq(missing_thumb_tag.id))
        .load::<(Entry, EntryTag)>(&conn)?;

    info!("found {} entries in need of thumbs", entries.len());
    for &(ref entry, ref _mapping) in &entries {
        let ext = entry.mime.as_ref()
            .unwrap()
            .splitn(2, "/")
            .skip(1).take(1)
            .next().unwrap();

        let path = PathBuf::new()
            .join(&content_store)
            .join(format!("f{}", &entry.hash[0..2]))
            .join(format!("{}.{}", &entry.hash[..], &ext));

        info!("path is => {:?}", path);
        aqua::util::processing::thumb_video(content_store, &entry.hash, &path)?;
    }

    Ok(())
}
