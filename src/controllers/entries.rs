use std::error::Error;
use std::path::PathBuf;

use controllers::prelude::*;
use models::queries;

use aqua_web::plug;
use aqua_web::mw::forms::MultipartForm;
use aqua_web::mw::router::Router;
use glob::glob;

static BASE_PATH: &'static str = "/Hydrus Network/db/client_files";

/// Fetch the file for a given entry ID
/// `GET /show/{id}`
pub fn show(conn: &mut plug::Conn) {
    let file_id = Router::param::<i64>(conn, "id")
        .expect("missing route param: id");

    match queries::find_entry(conn, file_id) {
        Ok(entry) => {
            let path_glob = format!("{}/f{}/{}.*",
                                    BASE_PATH,
                                    &entry.hash[0..2],
                                    &entry.hash);

            println!("glob pattern: {}", path_glob);
            let paths = glob(&path_glob)
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
            let path_glob = format!("{}/t{}/{}.*",
                                    BASE_PATH,
                                    &entry.hash[0..2],
                                    &entry.hash);

            println!("glob pattern: {}", path_glob);
            let paths = glob(&path_glob)
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
    use models::{queries, NewEntry}; 

    // TODO: simpler way to get extensions
    let mut form_fields = { conn.req_mut().mut_extensions().pop::<MultipartForm>() };
    let digest = form_fields.as_mut().and_then(|form| {
        extract_file(form, "upload").and_then(|file| hash_file(file.path))
    });

    let digest = match digest {
        None => { conn.send_resp(500, "file upload missing?"); return },
        Some(digest) => digest,
    };

    info!("got file digest: {}", digest);
    match queries::find_entry_by_hash(conn, &digest) {
        Ok(Some(entry)) => conn.send_resp(200, "TODO: serialize entry"),
        Ok(None)        => conn.send_resp(200, "TODO: write entry"),

        Err(msg) => conn.send_resp(500, &format!("could not load entry[{}]: {}", digest, msg)),
    };

}
