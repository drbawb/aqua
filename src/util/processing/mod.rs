use crypto::digest::Digest;
use crypto::sha2::Sha256;
use diesel;
use image;
use serde_json;
use std::{self, fmt, io};
use std::borrow::Borrow;
use std::convert::From;
use std::error::Error as StdError;
use std::result::Result as StdResult;
use std::fs::File;
use std::io::Read;
use std::path::Path;

mod image_detector;
mod video_detector;

// public detection & thumbnailing exports
pub use self::image_detector::mime_detect   as detect_image;
pub use self::image_detector::process_image as thumb_image;
pub use self::video_detector::ffmpeg_detect as detect_video;
pub use self::video_detector::process_video as thumb_video;

/// Reads a file from the specified path and returns its SHA256 digest.
pub fn hash_file(path: &Path) -> Result<String> {
    let mut buf = vec![];

    info!("path exists? {}",  (path.borrow()).exists());
    info!("path is file? {}", (path.borrow()).is_file());

    let digest = File::open(path)
         .and_then(|mut file| { file.read_to_end(&mut buf) })
         .map(|size| {

        debug!("read {} bytes into digest", size);
        let mut digest = Sha256::new();
        digest.input(&mut buf);
        digest.result_str()
    })?;

    Ok(digest)
}

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug)]
pub enum Error {
    DigestFailed,
    DetectionFailed,
    ThumbnailFailed,

    DbConnErr(diesel::ConnectionError),
    DbQueryErr(diesel::result::Error),
    IoErr(io::Error),
    ImageErr(image::ImageError),
    Misc(Box<StdError>),
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            // internal errors
            Error::DigestFailed      => "Unhandled error while generating SHA256 digest",
            Error::DetectionFailed   => "The file's type could not be detected",
            Error::ThumbnailFailed   => "The thumbnail could not be generated",

            // external errors
            Error::DbConnErr(ref inner)  => inner.description(),
            Error::DbQueryErr(ref inner) => inner.description(),
            Error::IoErr(ref inner)      => inner.description(),
            Error::ImageErr(ref inner)   => inner.description(),
            Error::Misc(ref inner)       => inner.description(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::DbConnErr(ref err)  => Some(err),
            Error::DbQueryErr(ref err) => Some(err),
            Error::IoErr(ref err)      => Some(err),
            Error::ImageErr(ref err)   => Some(err),
            Error::Misc(ref err)       => Some(err.as_ref()),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> StdResult<(), fmt::Error> {
        match *self {
            // TODO: better display impl.
            _ => write!(f, "{}", self.description()),
        }
    }
}

impl From<diesel::ConnectionError> for Error {
    fn from(err: diesel::ConnectionError) -> Self { Error::DbConnErr(err) }
}

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Self { Error::DbQueryErr(err) }
}

impl From<image::ImageError> for Error {
    fn from(err: image::ImageError) -> Self { Error::ImageErr(err) }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self { Error::IoErr(err) }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self { Error::Misc(Box::new(err)) }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Self { Error::Misc(Box::new(err)) }
}
