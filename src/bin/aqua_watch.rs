// This program watches a configured directory for incoming files.
// When a file is detected the following procedure is used to handle it:
//
//  * First we digest the file, and add the digest to our manifest.
//    This way we can intelligently handle future events for this file
//
//  * Next we attempt to detect a mime type using various heuristics, incl:
//
//    - Quick detection w/ our magic bytes table
//    - Querying external tools (e.g: ffmpeg, imagemagick, et al.)
//    - etc.
//
//  * If we cannot handle the mime type: the file is moved to an exception
//    directory, and an error is logged somewhere the user will see it.
//
//  * If we can handle the mime type: the file is moved to the content store
//    and an entry is created in the database.
//
//    - This should ideally be done atomically so that other aqua utility
//      processes (e.g: sister agnes) don't mistakenly mark the file as an
//      orphan / missing / etc.
//

#[macro_use] extern crate log;

extern crate aqua;
extern crate clap;
extern crate diesel;
extern crate dotenv;
extern crate env_logger;
extern crate image;
extern crate notify;
extern crate serde;
extern crate serde_json;

use aqua::models::{Entry, NewEntry};
use aqua::schema;
use aqua::util::processing;
use clap::{Arg, App};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use notify::{DebouncedEvent, Watcher, RecursiveMode, watcher};
use std::{env, fs};
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;

fn main() {
    dotenv().expect("must provide .env file, see README (TODO: haha jk)");
    env_logger::init().expect("could not initialize console logging");

    // read command line arguments
    let matches = App::new("aqua-watch")
        .version("0.1.0")
        .author("himechi <hime@localhost>")
        .about("Watches a directory for new files and moves them to the `aqua` content store.")
        .arg(Arg::with_name("INPUT")
             .help("Determines the input directory to be watched.")
             .required(true)
             .index(1))
        .arg(Arg::with_name("OUTPUT")
             .help("The root of the aqua content store where files will be moved.")
             .required(true)
             .index(2))
        .get_matches();


    let dropbox_path  = matches.value_of("INPUT").unwrap();
    let content_store = matches.value_of("OUTPUT").unwrap();

    // setup fs watcher
    let (fs_tx, fs_rx) = channel();
    let mut fs_watcher = watcher(fs_tx, Duration::from_millis(1000))
        .expect("could not create file system watcher!");

    fs_watcher.watch(dropbox_path, RecursiveMode::NonRecursive)
        .expect("could not enroll dropbox in fs events queue");

    // TODO: needs to wait for file to be completely written before attempting
    //       to digest it ...
    // process filesystem events ...
    loop {
        match fs_rx.recv() {
            Ok(DebouncedEvent::Create(path)) => {
                if path.is_file() { 
                    match handle_new_file(path, content_store) {
                        Ok(_res) => info!("file processed successfully ..."),
                        Err(msg) => warn!("could not process file: {:?} (inner: {:?})", msg, msg.cause()),
                    };
                }
                else { info!("directory created, ignoring ..."); }
            },
            Ok(event) => info!("unhandled evt: {:?}", event),
            Err(msg) => warn!("fs err: {}", msg),
        }
    }
}

// TODO: check that file doesn't exist before moving it ...
fn handle_new_file(path: PathBuf, content_store: &str) -> processing::Result<()> {
    let digest = aqua::util::processing::hash_file(path.as_path())?;
    let mut file = OpenOptions::new()
        .read(true)
        .write(false)
        .create_new(false)
        .open(path.as_path())?;

    // read file into memory
    let mut buf = vec![];
    file.read_to_end(&mut buf)?;

    // TODO: move_file() & db() is probably going to be common to all handlers?
    if let Some(image_metadata) = aqua::util::processing::detect_image(&buf) {
        info!("got an image ...");
        aqua::util::processing::thumb_image(content_store, &digest, &buf)?;
        move_file(path.as_path(), content_store, &digest, image_metadata.extension())?;

        let db_entry = create_db_entry(&digest, image_metadata.mime())?;
        info!("inserted: {:?} into database", db_entry);

        Ok(())
    } else if let Some(ffmpeg_metadata) = aqua::util::processing::detect_video(path.as_path())? {
        info!("got an video ...");
        aqua::util::processing::thumb_video(content_store, &digest, &path)?;
        move_file(path.as_path(), content_store, &digest, ffmpeg_metadata.ext)?;

        let db_entry = create_db_entry(&digest, ffmpeg_metadata.mime)?;
        info!("inserted: {:?} into database", db_entry);

        Ok(())
    } else {
        Err(processing::Error::DetectionFailed)
    }
}

fn establish_connection() -> processing::Result<PgConnection> {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL not set in `.env` file !!!");

    Ok(PgConnection::establish(&database_url)?)
}

// create entry in database
fn create_db_entry(digest: &str, mime_ty: &str) -> processing::Result<Entry> {
    let pg_conn = establish_connection()?;
    let aqua_entry = NewEntry { hash: &digest, mime: Some(&mime_ty) };
    let entry = diesel::insert(&aqua_entry)
        .into(schema::entries::table)
        .get_result(&pg_conn);

    Ok(entry?)
}

// moves the file from `src_path` to the `content_store` based on its digest
fn move_file(src_path: &Path, content_store: &str, digest: &str, file_ext: &str) -> processing::Result<()> {
    // carve out a bucket based on first byte of SHA256 digest
    // create the bucket if it does not exist
    let file_bucket    = format!("f{}", &digest[0..2]);
    let file_filename  = format!("{}.{}", &digest, file_ext);

    // create destination path
    let dest = PathBuf::from(content_store)
        .join(file_bucket)
        .join(file_filename);

    // TODO: bad error type ... 
    let bucket_dir = dest.parent().ok_or(processing::Error::ThumbnailFailed)?;
    fs::create_dir_all(bucket_dir)?;

    // move the file 
    Ok(fs::rename(src_path, &dest)?)
}
