use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use controllers::prelude::*;
use models;
use views;

use aqua_web::plug;
use aqua_web::mw::forms::{MultipartForm, FormField, SavedFile};
use crypto::digest::Digest;
use crypto::sha2::Sha256;

#[derive(Serialize)]
struct Wat {
    derp: String,
}

/// Does the thing, wins the points ...
pub fn index(conn: &mut plug::Conn) {
    // db lulz
    let entries = ::models::queries::all_entries(conn.req());
    println!("got entries: {:?}", entries);

    // render template
    let data = Wat { derp: format!("entry => {:?}", entries) };
    let view = views::render_into(conn.req(), "layouts/main", "dash/index", &data);

    conn.send_resp(200, &view);
}

/// Extracts a file from a multipart form if the key exists & it is a file
fn extract_file(form: &mut MultipartForm, field: &str) -> Option<SavedFile> {
    match form.entries.remove(field) {
        Some(FormField::File(file)) => Some(file),
        Some(_) => { warn!("file expected, but got string"); None },
        None    => { warn!("file expected, but not present"); None },
    }
}

pub fn submit(conn: &mut plug::Conn) {

    // 
    let mut form_fields = { conn.req_mut().mut_extensions().pop::<MultipartForm>() };
    let digest = form_fields.as_mut().and_then(|form| {
        extract_file(form, "upload").and_then(|file| hash_file(file.path))
    });

    match digest {
        Some(digest) => conn.send_resp(200, &format!("nice file fam: {}", digest)),
        None         => conn.send_resp(500, "yo where is my file fam?"),
    }
}

fn hash_file<P: AsRef<Path>>(path: P) -> Option<String> {
    println!("file was pretty coo, gonna hash it");
    let mut buf = vec![];

    info!("path exists? {}",  (path.as_ref()).exists());
    info!("path is file? {}", (path.as_ref()).is_file());

    File::open(path)
         .and_then(|mut file| { file.read_to_end(&mut buf) })
         .map(|size| {

        println!("read {} bytes", size);
        let mut digest = Sha256::new();
        digest.input(&mut buf);
        digest.result_str()
    }).ok()
}
