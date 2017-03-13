use image::{self, ImageFormat};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

/// ImageMeta stores mappings of common image filetypes to their associated
/// MIME type and typical file extension. This is useful in processing files 
/// which either have no filename, or files where the filename provided by the 
/// client is considered "untrusted."
///
/// At the moment image detection is done using a table of offsets & magic bytes.
/// Only formats supported by the `image` library are detected.
#[derive(Copy, Clone)]
pub struct ImageMeta {
   ext: &'static str,
   fmt: ImageFormat,
}

impl ImageMeta {
    // NOTE: sorry, this table *must* be built at compile time.
    //       I don't feel like fighting the borrowck on this one.
    pub fn from(extension: &'static str, format: ImageFormat) -> ImageMeta {
        ImageMeta {
            ext: extension,
            fmt: format,
        }
    }

    /// Fetches this image's file extension
    pub fn extension(&self) -> &'static str { self.ext }

    /// Fetches this image's format enumeration
    pub fn format(&self) -> ImageFormat { self.fmt }

    /// Fetches the MIME type (for use w/ content-type)
    pub fn mime(&self) -> &'static str {
        match self.fmt {
            ImageFormat::BMP  => "image/bmp",
            ImageFormat::GIF => "image/gif",
            ImageFormat::HDR  => "image/x-hdr",
            ImageFormat::ICO  => "image/x-icon",
            ImageFormat::JPEG => "image/jpeg",
            ImageFormat::PNG => "image/png",
            ImageFormat::PPM => "image/x-portable-pixmap",
            ImageFormat::TGA => "image/tga",
            ImageFormat::TIFF => "image/tiff",
            ImageFormat::WEBP => "image/webp",
        }
    }
}

// TODO: moar formats, MOAR!
pub fn mime_detect(data: &[u8]) -> Option<ImageMeta> {
    // OFFSET   MATCHER             MIME_TYPE
    let mime_table: Vec<(usize, &'static [u8], ImageMeta)> = vec![
        (0,     &b"BM"[..],         ImageMeta::from("bmp",  ImageFormat::BMP)  ),
        (0,     &b"GIF87a"[..],     ImageMeta::from("gif",  ImageFormat::GIF)  ),
        (0,     &b"GIF89a"[..],     ImageMeta::from("gif",  ImageFormat::GIF)  ),
        (0,     &b"#?RADIANCE"[..], ImageMeta::from("hdr",  ImageFormat::HDR)  ),
        (0,     &b"\0\0\x01\0"[..], ImageMeta::from("ico",  ImageFormat::ICO)  ),
        (0,     &b"\xff\xd8"[..],   ImageMeta::from("jpg",  ImageFormat::JPEG) ),
        (0,     &b"\x89PNG"[..],    ImageMeta::from("png",  ImageFormat::PNG)  ),
        (0,     &b"MM.*"[..],       ImageMeta::from("tiff", ImageFormat::TIFF) ),
        (0,     &b"II*."[..],       ImageMeta::from("tiff", ImageFormat::TIFF) ),
    ];

    // see if file matches a header descriptor we know...
    for &(offset, matcher, file_ty) in &mime_table {
        if data[offset..].starts_with(matcher) { return Some(file_ty) }
    }

    None
}

// creates a thumbnail in the content store for the specified digest
// this expects an `ImageMeta` structure describing the input.
pub fn process_image(content_store: &str, digest: &str, buf: &[u8]) -> super::Result<()> {
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
    let bucket_dir = dest.parent().ok_or(super::Error::ThumbnailFailed)?;
    fs::create_dir_all(bucket_dir)?;
    let mut dest_file = File::create(&dest)?;


    {
        // HACK: force image to be saved as rgba, grayscale jpg thumbnails are broken
        // (for some reason they come out all white)
        let rgba_image = thumb.to_rgba();
        let bytes = rgba_image.into_raw();
        let mut encoder = ::image::jpeg::JPEGEncoder::new(&mut dest_file);
        encoder.encode(&bytes, 200, 200, ::image::ColorType::RGBA(8))?;
    }

    // thumb.save(&mut dest_file, image::ImageFormat::JPEG)?;
    Ok(dest_file.flush()?)
}
