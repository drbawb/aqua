use std::collections::HashMap;
use plug::{Conn, Plug};

use multipart::server::{Multipart, MultipartField, SaveResult};
pub use multipart::server::{SaveDir, SavedFile};

pub enum FormField {
    Value(String),
    File(SavedFile),
}

pub struct MultipartForm {
    pub entries: HashMap<String, FormField>,
    pub save_dir: SaveDir
}

/// This middleware looks for incoming requests w/ `content-type: multipart/form-data`
/// The boundary is extracted from the header, and then the request body is interpreted
/// as multipart form data and stored in the request extensions.
pub struct MultipartParser;

impl Plug for MultipartParser {
    fn call(&self, conn: &mut Conn) {
        let boundary = { // borrow request to find content-type header
            conn.req().headers().find("content-type")
                .and_then(|header| header.into_iter()
                                     .find(|field| field.starts_with("multipart/form-data")))
                .and_then(|header| header.split("; boundary=").nth(1))
                .map(|boundary| boundary.to_string())
        };
                

        if let Some(boundary) = boundary {
            info!("found multipart boundary, HANDLE IT!");
            let mut mp_files = { // borrow request mutably to read body
                let mut mp_data = Multipart::with_body(conn.req_mut().body(), boundary);
                let mut mp_files = HashMap::new();

                // attempt to save all files to temp storage
                // TODO: when is this storage purged?
                let entry_dir = match mp_data.save_all() {
                    SaveResult::Full(entries) => {
                        for (key,val) in entries.fields {
                            mp_files.insert(key, FormField::Value(val));
                        }

                        for (key,val) in entries.files {
                            mp_files.insert(key, FormField::File(val));
                        }

                        entries.dir
                    },

                    // TODO: handle multipart failures
                    _ => panic!("error reading multipart formdata"),
                };

                MultipartForm {
                    entries:  mp_files,
                    save_dir: entry_dir,
                }
            };

            // now we can borrow request again to insert the processed formdata
            conn.req_mut().mut_extensions().insert::<MultipartForm>(mp_files);
        }
    }
}
