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

use aqua::controllers::prelude::hash_file;
use aqua::models::{Entry, NewEntry};
use aqua::schema;
use clap::{Arg, App};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use notify::{DebouncedEvent, Watcher, RecursiveMode, watcher};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
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

    // process filesystem events ...
    loop {
        match fs_rx.recv() {
            Ok(DebouncedEvent::Create(path)) => {
                if path.is_file() { handle_new_file(path, content_store) }
                else { info!("directory created, ignoring ..."); }
            },
            Ok(event) => info!("unhandled evt: {:?}", event),
            Err(msg) => warn!("fs err: {}", msg),
        }
    }
}

fn handle_new_file(path: PathBuf, content_store: &str) {
    let digest = hash_file(path.as_path())
        .expect("could not get digest for file (!!!)");

    let mut file = OpenOptions::new()
        .read(true)
        .write(false)
        .create_new(false)
        .open(path.as_path())
        .expect("could not open image file");

    // read file into memory
    let mut buf = vec![];
    file.read_to_end(&mut buf).expect("could not read file");


    let (mime_ty, file_ext) = match aqua::util::mime_detect(&buf) {
        Some(file_ty) => {
            process_image(content_store, &digest, &buf);
            (file_ty.mime().to_string(), file_ty.extension().to_string())
        },

        None => {
            // TODO: thumbnail a video, properly figure out mimety
            let file_extension = path.extension()
                .unwrap()
                .to_string_lossy();

            (format!("video/{}", file_extension), file_extension.into_owned())
            
        },
    };

    // carve out a bucket based on first byte of SHA256 digest
    // create the bucket if it does not exist
    let file_bucket    = format!("f{}", &digest[0..2]);
    let file_filename  = format!("{}.{}", &digest, file_ext);

    // move file to content store
    let dest = PathBuf::from(content_store)
        .join(file_bucket)
        .join(file_filename);

    fs::create_dir_all(dest.parent().unwrap()).expect("could not create file bucket");   
    fs::rename(path, dest)
        .expect("could not move file to content store");

    // create entry in database
    let pg_conn = establish_connection();
    let aqua_entry = NewEntry { hash: &digest, mime: Some(&mime_ty) };
    let entry: Result<Entry, diesel::result::Error> = diesel::insert(&aqua_entry)
        .into(schema::entries::table)
        .get_result(&pg_conn);

    match entry {
        Ok(_entry) => info!("entry added to database: {}", digest),
        Err(msg) => warn!("could not store entry in database: {}", msg),
    };
}

pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL not set");

    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

// creates a thumbnail in the content store for the specified digest
// this expects an `ImageMeta` structure describing the input.
fn process_image(content_store: &str, digest: &str, buf: &[u8]) {
    // create in memory thumbnail
    let image = image::load_from_memory(&buf)
        .expect("could not read image into memory");

    let thumb = image.resize(200, 200, image::FilterType::Nearest);
    let thumb_bucket   = format!("t{}", &digest[0..2]);
    let thumb_filename = format!("{}.thumbnail", &digest);
    // store them in content store

    let dest = PathBuf::from(content_store)
        .join(thumb_bucket)
        .join(thumb_filename);

    // write thumbnail file
    fs::create_dir_all(dest.parent().unwrap()).expect("could not create thumbnail bucket");
    let mut dest_file = File::create(dest)
        .expect("could not create thumbnail in content store");

    thumb.save(&mut dest_file, image::ImageFormat::JPEG)
        .expect("could not write to thumbnail in content store"); 

    dest_file.flush().expect("could not flush thumbnail to disk");
}
