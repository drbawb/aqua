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
use std::{env, fmt};
use std::convert::From;
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;

#[derive(Debug)]
enum ProcessingError {
    DigestFailed,
    DetectionFailed,
    ThumbnailFailed,
    DbConnErr(diesel::ConnectionError),
    DbQueryErr(diesel::result::Error),
    IoErr(io::Error),
    ImageErr(image::ImageError),
}

impl Error for ProcessingError {
    fn description(&self) -> &str {
        match *self {
            // internal errors
            ProcessingError::DigestFailed      => "Unhandled error while generating SHA256 digest",
            ProcessingError::DetectionFailed   => "The file's type could not be detected",
            ProcessingError::ThumbnailFailed   => "The thumbnail could not be generated",

            // external errors
            ProcessingError::DbConnErr(ref inner)  => inner.description(),
            ProcessingError::DbQueryErr(ref inner) => inner.description(),
            ProcessingError::IoErr(ref inner)      => inner.description(),
            ProcessingError::ImageErr(ref inner)   => inner.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            ProcessingError::DbConnErr(ref err)  => Some(err),
            ProcessingError::DbQueryErr(ref err) => Some(err),
            ProcessingError::IoErr(ref err)      => Some(err),
            ProcessingError::ImageErr(ref err)   => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for ProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            // TODO: better display impl.
            _ => write!(f, "{}", self.description()),
        }
    }
}

impl From<diesel::ConnectionError> for ProcessingError {
    fn from(err: diesel::ConnectionError) -> Self { ProcessingError::DbConnErr(err) }
}

impl From<diesel::result::Error> for ProcessingError {
    fn from(err: diesel::result::Error) -> Self { ProcessingError::DbQueryErr(err) }
}

impl From<image::ImageError> for ProcessingError {
    fn from(err: image::ImageError) -> Self { ProcessingError::ImageErr(err) }
}

impl From<io::Error> for ProcessingError {
    fn from(err: io::Error) -> Self { ProcessingError::IoErr(err) }
}

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
                if path.is_file() { 
                    match handle_new_file(path, content_store) {
                        Ok(_res) => info!("file processed successfully ..."),
                        Err(msg) => warn!("could not process file: {}", msg.description()),
                    };
                }
                else { info!("directory created, ignoring ..."); }
            },
            Ok(event) => info!("unhandled evt: {:?}", event),
            Err(msg) => warn!("fs err: {}", msg),
        }
    }
}

fn handle_new_file(path: PathBuf, content_store: &str) -> Result<(), ProcessingError> {
    let digest = hash_file(path.as_path())
        .ok_or(ProcessingError::DigestFailed)?;

    let mut file = OpenOptions::new()
        .read(true)
        .write(false)
        .create_new(false)
        .open(path.as_path())?;

    // read file into memory
    let mut buf = vec![];
    file.read_to_end(&mut buf)?;

    if let Some(image_metadata) = aqua::util::mime_detect(&buf) {
        info!("got an image ...");
        process_image(content_store, &digest, &buf)?;
        move_file(path.as_path(), content_store, &digest, image_metadata.extension())?;

        let db_entry = create_db_entry(&digest, image_metadata.mime())?;
        info!("inserted: {:?} into database", db_entry);

        Ok(())
    } else if let Some(_nil) = ffmpeg_detect(path.as_path()) {
        unreachable!()
    } else {
        Err(ProcessingError::DetectionFailed)
    }
}

fn ffmpeg_detect(path: &Path) -> Option<()> {
    None
}

fn establish_connection() -> Result<PgConnection, ProcessingError> {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL not set in `.env` file !!!");

    Ok(PgConnection::establish(&database_url)?)
}

// create entry in database
fn create_db_entry(digest: &str, mime_ty: &str) -> Result<Entry, ProcessingError> {
    let pg_conn = establish_connection()?;
    let aqua_entry = NewEntry { hash: &digest, mime: Some(&mime_ty) };
    let entry: Result<Entry, diesel::result::Error> = diesel::insert(&aqua_entry)
        .into(schema::entries::table)
        .get_result(&pg_conn);

    Ok(entry?)
}

// moves the file from `src_path` to the `content_store` based on its digest
fn move_file(src_path: &Path, content_store: &str, digest: &str, file_ext: &str) -> Result<(), ProcessingError> {
    // carve out a bucket based on first byte of SHA256 digest
    // create the bucket if it does not exist
    let file_bucket    = format!("f{}", &digest[0..2]);
    let file_filename  = format!("{}.{}", &digest, file_ext);

    // create destination path
    let dest = PathBuf::from(content_store)
        .join(file_bucket)
        .join(file_filename);

    // TODO: bad error type ... 
    let bucket_dir = dest.parent().ok_or(ProcessingError::ThumbnailFailed)?;
    fs::create_dir_all(bucket_dir)?;

    // move the file 
    Ok(fs::rename(src_path, &dest)?)
}

// creates a thumbnail in the content store for the specified digest
// this expects an `ImageMeta` structure describing the input.
fn process_image(content_store: &str, digest: &str, buf: &[u8]) -> Result<(), ProcessingError> {
    // create in memory thumbnail
    let image = image::load_from_memory(&buf)?;

    let thumb = image.resize(200, 200, image::FilterType::Nearest);
    let thumb_bucket   = format!("t{}", &digest[0..2]);
    let thumb_filename = format!("{}.thumbnail", &digest);
    
    // store them in content store
    let dest = PathBuf::from(content_store)
        .join(thumb_bucket)
        .join(thumb_filename);

    // write thumbnail file to disk
    fs::create_dir_all(dest.parent().unwrap())?;
    let mut dest_file = File::create(dest)?;
    thumb.save(&mut dest_file, image::ImageFormat::JPEG)?;
    Ok(dest_file.flush()?)
}
