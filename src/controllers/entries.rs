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

pub fn submit(conn: &mut plug::Conn) {
    use models::{queries, NewEntry}; 

    // TODO: simpler way to get extensions
    let mut form_fields = { conn.req_mut().mut_extensions().pop::<MultipartForm>() };
    let digest = form_fields.as_mut().and_then(|form| {
        extract_file(form, "upload").and_then(|file| hash_file(file.path))
    });

    if let Some(digest) = digest {
        let entry = queries::find_or_insert(conn, NewEntry { hash: &digest, mime: None });
        conn.send_resp(200, &format!("nice file fam: {:?}", entry))
    } else { conn.send_resp(500, "yo where is my file fam?") }

    // store the digest (if it does not exist)

    // save the file (if new)
    // respond with (hash, tags)


}
