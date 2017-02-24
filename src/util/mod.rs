use image::ImageFormat;

pub mod db;
pub mod template;
pub mod timer;
pub mod try_file;

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
