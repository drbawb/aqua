use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::PathBuf;

use controllers::prelude::*;
use models::{queries, Tag};
use views;

use aqua_web::plug;
use aqua_web::mw::forms::{MultipartForm, SavedFile};
use aqua_web::mw::router::Router;
use glob::glob;
use serde_json;

#[derive(Serialize)]
struct TagView {
    tags: Vec<Tag>,
}

fn glob_for_category(category: &str, digest: &str) -> String {
    // TODO: assert digest is really a digest
    // TODO: assert category is really a category

    PathBuf::from(env::var("CONTENT_STORE").unwrap())
        .join(format!("{}{}", category, &digest[..2]))
        .join(&digest)
        .with_extension("*")
        .to_string_lossy()
        .into_owned()
}

/// Fetch the file for a given entry ID
/// `GET /show/{id}`
pub fn show(conn: &mut plug::Conn) {
    let file_id = Router::param::<i64>(conn, "id")
        .expect("missing route param: id");

    match queries::find_entry(conn, file_id) {
        Ok(entry) => {
            let glob_pattern = glob_for_category("f", &entry.hash);
            info!("glob pattern: {}", glob_pattern);

            let paths = glob(&glob_pattern)
                .expect("could not parse glob pattern")
                .map(|res| res.ok().unwrap())
                .collect::<Vec<PathBuf>>();

            assert_eq!(paths.len(), 1);
            conn.send_file(200, &paths[0]);
            // conn.send_resp(200, &path_glob);
        },

        Err(err) => conn.send_resp(404, err.description()),
    }
}

pub fn show_thumb(conn: &mut plug::Conn) {
    let file_id = Router::param::<i64>(conn, "id")
        .expect("missing route param: id");

    match queries::find_entry(conn, file_id) {
        Ok(entry) => {
            let glob_pattern = glob_for_category("t", &entry.hash);
            info!("glob pattern: {}", glob_pattern);

            let paths = glob(&glob_pattern)
                .expect("could not parse glob pattern")
                .map(|res| res.ok().unwrap())
                .collect::<Vec<PathBuf>>();

            assert_eq!(paths.len(), 1);
            conn.send_file(200, &paths[0]);
            // conn.send_resp(200, &path_glob);
        },

        Err(err) => conn.send_resp(404, err.description()),
    }
}

/// `GET /entries/{id}/tags`
///
/// Gets a view fragment to show and modify the tags.
pub fn show_entry_tags(conn: &mut plug::Conn) {
    let entry_id = Router::param::<i64>(conn, "id")
        .expect("missing route param: id");

    let tags = queries::find_tags_for(conn, entry_id)
        .expect("could not load tags");

    let data = TagView { tags: tags };
    let view = views::render(conn.req(), "tag/_panel", &data);
    conn.send_resp(200, &view);
}

/// `POST /entries/upload`
///
/// Returns a `models::Entry` as JSON or an HTTP 500 error on failure.
/// Expects a multipart form containing a file payload in the field `upload`.
/// This payload is extracted and converted to a SHA-256 digest.
///
/// If the entry already exists it is returned immediately, otherwise it is
/// moved to the content addressable storage pool and the entry is created.
///
pub fn submit(conn: &mut plug::Conn) {
    use models::queries;

    // TODO: simpler way to get extensions
    let mut form_fields = { conn.req_mut().mut_extensions().pop::<MultipartForm>() };
   
    // NOTE: these are separate b/c we need to hang on to the file ref ...
    let file_upload = form_fields.as_mut()
        .and_then(|form| extract_file(form, "upload"));

    let digest = file_upload.as_ref()
            .and_then(|file| hash_file(file.path.as_path()));

    // TODO: these are gross b/c we can't return anything, thus there's no good
    //       way to use Result+`try!` ...
    let file_upload = match file_upload {
        None => { conn.send_resp(500, "file upload missing?"); return },
        Some(file_upload) => file_upload,
    };

    let digest = match digest {
        None => { conn.send_resp(500, "file upload did not digest?"); return },
        Some(digest) => digest,
    };

    info!("got file digest: {}", digest);
    match queries::find_entry_by_hash(conn, &digest) {
        Ok(Some(entry)) => send_json(conn, entry),
        Ok(None)        => write_entry(conn, digest, file_upload),

        Err(msg) => conn.send_resp(500, &format!("could not load entry[{}]: {}", digest, msg)),
    };

}

// TODO: pull this out to aqua web?
fn send_json<T: ::serde::ser::Serialize>(conn: &mut plug::Conn, json_payload: T) {
    let output = serde_json::to_string(&json_payload)
        .expect("could not serialize output!");

    conn.send_resp(200, &output);
}

// TODO: ???
fn write_entry(conn: &mut plug::Conn, digest: String, file: SavedFile) {
    use models::{queries, NewEntry}; 

    // open the file
    let mut file = match File::open(file.path) {
        Ok(file) => file,
        Err(_msg) => { conn.send_resp(500, "could not open your upload..."); return },
    };

    // read into temp buffer
    let mut buf = vec![];
    let file_ty = match file.read_to_end(&mut buf) {
        Ok(_size) => mime_detect(&buf[..]),
        Err(_msg) => { conn.send_resp(500, "could not read your upload..."); return },
    };

    // create content aware address for it
    let (content_path, content_name) = match file_ty {
        Some(file_ty) => (
            format!("{}/f{}", env::var("CONTENT_STORE").unwrap(), &digest[..2]),
            format!("{}.{}", &digest[..], file_ty)
        ),

        None => { conn.send_resp(500, "unsupported mime type"); return },
    };

    // create bucket in content store
    let dst_path = PathBuf::from(content_path);
    if let Err(msg) = fs::create_dir_all(&dst_path) {
        warn!("could not create content store bucket: {}", msg);
        conn.send_resp(500, "could not add file to content store");
        return
    }

    // copy file to bucket
    let dst_file_name = dst_path.join(content_name);
    let dst_file_copy = File::create(dst_file_name).and_then(|mut file| {
       io::copy(&mut Cursor::new(buf), &mut file)
    });

    if let Err(msg) = dst_file_copy {
        warn!("error storing file: {:?}", msg);
        conn.send_resp(500, "could not add file to content store"); 
        return
    }

    // store that sucker in the db ...
    match queries::find_or_insert(conn, NewEntry { hash: &digest, mime: file_ty }) {
        Some(entry) => send_json(conn, entry),
        None=> conn.send_resp(500, "could not store entry in DB"),
    }
}

// TODO: moar formats, MOAR!
fn mime_detect(data: &[u8]) -> Option<&'static str> {
    // OFFSET   MATCHER     MIME_TYPE
    let mime_table = vec![
        (0,     &b"BM"[..],          "bmp"),
        (0,     &b"GIF87a"[..],      "gif"),
        (0,     &b"GIF89a"[..],      "gif"),
        (0,     &b"\xff\xd8"[..],    "jpg"),
        (0,     &b"\x89PNG"[..],     "png"),
    ];

    // see if file matches a header descriptor we know...
    for &(offset, matcher, file_ty) in &mime_table {
        if data[offset..].starts_with(matcher) { return Some(file_ty) }
    }

    None
}
