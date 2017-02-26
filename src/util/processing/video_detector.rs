use serde_json;
use std::{fs, process};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use super::ProcessingError;

#[derive(Debug, Serialize, Deserialize)]
pub struct FFProbeResult {
    format: Option<FFProbeFormat>,
    streams: Option<Vec<FFProbeStream>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FFProbeFormat {
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
pub struct FFProbeStream {
    codec_name: String,
    codec_type: String,
}

#[derive(Debug)]
pub struct FFProbeMeta {
    pub mime: &'static str,
    pub ext:  &'static str,
}

/// This function uses the system installation of `ffprobe` to detect the following:
///   - Does the file have (at least) one video stream?
///   - Which container format was detected?
///
/// The container format is then mapped to a common mime & extension which is used
/// by other parts of the `aqua` application suite to determine how an asset should
/// be displayed.
pub fn ffmpeg_detect(path: &Path) -> Result<Option<FFProbeMeta>, ProcessingError> {
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

pub fn process_video(content_store: &str, digest: &str, src: &Path) -> super::ProcessingResult<()> {
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
    let bucket_dir = dest.parent().ok_or(ProcessingError::ThumbnailFailed)?;
    fs::create_dir_all(bucket_dir)?;
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
