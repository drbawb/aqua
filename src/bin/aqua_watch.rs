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
#[macro_use] extern crate serde_derive;

extern crate aqua;
extern crate clap;
extern crate diesel;
extern crate dotenv;
extern crate env_logger;
extern crate image;
extern crate notify;
extern crate serde;
extern crate serde_json;

use aqua::controllers::prelude::hash_file;
use aqua::models::{Entry, NewEntry};
use aqua::schema;
use clap::{Arg, App};
use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use notify::{DebouncedEvent, Watcher, RecursiveMode, watcher};
use std::{env, fmt, process};
use std::collections::HashMap;
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
    Misc(Box<Error>),
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
            ProcessingError::Misc(ref inner)       => inner.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            ProcessingError::DbConnErr(ref err)  => Some(err),
            ProcessingError::DbQueryErr(ref err) => Some(err),
            ProcessingError::IoErr(ref err)      => Some(err),
            ProcessingError::ImageErr(ref err)   => Some(err),
            ProcessingError::Misc(ref err)       => Some(err.as_ref()),
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

impl From<serde_json::Error> for ProcessingError {
    fn from(err: serde_json::Error) -> Self { ProcessingError::Misc(Box::new(err)) }
}

impl From<std::string::FromUtf8Error> for ProcessingError {
    fn from(err: std::string::FromUtf8Error) -> Self { ProcessingError::Misc(Box::new(err)) }
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

    // TODO: move_file() & db() is probably going to be common to all handlers?
    if let Some(image_metadata) = aqua::util::mime_detect(&buf) {
        info!("got an image ...");
        process_image(content_store, &digest, &buf)?;
        move_file(path.as_path(), content_store, &digest, image_metadata.extension())?;

        let db_entry = create_db_entry(&digest, image_metadata.mime())?;
        info!("inserted: {:?} into database", db_entry);

        Ok(())
    } else if let Some(ffmpeg_metadata) = ffmpeg_detect(path.as_path())? {
        info!("got an video ...");
        process_video(content_store, &digest, &path)?;
        move_file(path.as_path(), content_store, &digest, ffmpeg_metadata.ext)?;

        let db_entry = create_db_entry(&digest, ffmpeg_metadata.mime)?;
        info!("inserted: {:?} into database", db_entry);

        Ok(())
    } else {
        Err(ProcessingError::DetectionFailed)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct FFProbeResult {
    format: Option<FFProbeFormat>,
    streams: Option<Vec<FFProbeStream>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FFProbeFormat {
    filename:    String,
    nb_streams:  i32,
    format_name: String,
    start_time:  String, // NOTE: these appear to be fixed decimal
    duration:    String, // NOTE: these appear to be fixed decimal
    size:        String, // NOTE: appears to be an integer
    bit_rate:    String, // NOTE: appears to be an integer
    probe_score: i32,    // NOTE: accuracy of the detection? (???)

    tags: HashMap<String, String>, // NOTE: not sure this is correct type
}

#[derive(Debug, Serialize, Deserialize)]
struct FFProbeStream {
    codec_name: String,
    codec_type: String,
}

struct FFProbeMeta {
    pub mime: &'static str,
    pub ext:  &'static str,
}

fn ffmpeg_detect(path: &Path) -> Result<Option<FFProbeMeta>, ProcessingError> {
    let ffprobe_cmd = process::Command::new("ffprobe")
        .arg("-v").arg("quiet")            // silence debug output
        .arg("-hide_banner")               // don't print ffmpeg configuration
        .arg("-print_format").arg("json") // serialize to json
        .arg("-show_format")              // display format data
        .arg("-show_streams")             // display stream data
        .arg("-i").arg(path.as_os_str())  // set the input to current file
        .output()?;

    let json_str = String::from_utf8(ffprobe_cmd.stdout)?;
    let probe_result: FFProbeResult = serde_json::from_str(&json_str)?;
    info!("got result: {:?}", probe_result);

    // see if ffprobe was able to determine the file type ...
    let probe_format = match probe_result.format {
        Some(format_data) => format_data,
        None => return Ok(None),
    };

    let probe_streams = match probe_result.streams {
        Some(stream_data) => stream_data,
        None => return Ok(None),
    };

    // welp ... guess there's nothing to thumbnail (!!!)
    info!("got format: {:?}", probe_format);
    info!("got streams: {:?}", probe_streams);

    let number_of_videos = probe_streams
        .iter()
        .filter(|el| el.codec_type == "video")
        .count();

    if number_of_videos <= 0 { return Err(ProcessingError::DetectionFailed) }

    // TODO: how do we correlate format_name w/ stream & stream position?
    // TODO: I believe this should be matching on containers (which is what will be moved
    //       to the content store; and therefore what will be played back ...)
    //      
    let meta_data = if probe_format.format_name.contains("matroska") {
        Some(FFProbeMeta { mime: "video/x-matroska", ext: "mkv" })
    } else if probe_format.format_name.contains("mp4") {
        Some(FFProbeMeta { mime: "video/mp4", ext: "mp4" })
    } else { None };
    
    Ok(meta_data)
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

fn process_video(content_store: &str, digest: &str, src: &Path) -> Result<(), ProcessingError> {
    let thumb_bucket   = format!("t{}", &digest[0..2]);
    let thumb_filename = format!("{}.thumbnail", &digest);

    // store them in content store
    let dest = PathBuf::from(content_store)
        .join(thumb_bucket)
        .join(thumb_filename);

    // TODO: seems weird to have "mjpeg" in here... but I couldn't find any other
    //       JPEG muxer/encoder in my ffmpeg install ...
    //
    // write thumbnail file to disk
    fs::create_dir_all(dest.parent().unwrap())?;
    let ffmpeg_cmd = process::Command::new("ffmpeg")
        .arg("-i").arg(src.as_os_str())            // the input file
        .arg("-vf").arg("thumbnail,scale=200:200") // have ffmpeg seek for a "thumbnail"
        .arg("-frames:v").arg("1")                 // take a single frame
        .arg("-f").arg("mjpeg")                    // save it as jpeg
        .arg(dest.as_path().as_os_str())           // into the content store
        .output()?;

    debug!("ffmpeg stderr: {}", String::from_utf8_lossy(&ffmpeg_cmd.stderr));
    debug!("ffmpeg stdout: {}", String::from_utf8_lossy(&ffmpeg_cmd.stdout));

    info!("digest? {}", digest);
    info!("dest exists? {:?} => {}", dest, dest.is_file());

    match dest.is_file() {
        true  => Ok(()),
        false => Err(ProcessingError::ThumbnailFailed),
    }
}
