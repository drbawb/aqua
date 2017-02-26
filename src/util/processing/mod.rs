use crypto::digest::Digest;
use crypto::sha2::Sha256;
use diesel;
use image;
use serde_json;
use std::{self, fmt, io};
use std::borrow::Borrow;
use std::convert::From;
use std::error::Error;
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
pub fn hash_file(path: &Path) -> ProcessingResult<String> {
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



pub type ProcessingResult<T> = Result<T, ProcessingError>;

#[derive(Debug)]
pub enum ProcessingError {
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
