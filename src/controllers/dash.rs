use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use controllers::prelude::*;
use models;
use views;

use aqua_web::plug;
use aqua_web::mw::forms::{MultipartForm, FormField, SavedFile};
use aqua_web::mw::route::MatchContext;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use glob::glob;

static BASE_PATH: &'static str = "/Hydrus Network/db/client_files";

#[derive(Serialize)]
struct Wat {
    derp: String,
}

#[derive(Serialize)]
struct EntryListView {
    entries: Vec<models::EntryTag>,
}

/// Does the thing, wins the points ...
pub fn index(conn: &mut plug::Conn) {
    // db lulz
    let entries = ::models::queries::all_entries(conn);
    println!("got entries: {:?}", entries);

    // render template
    let data = Wat { derp: format!("entry => {:?}", entries) };
    let view = views::render_into(conn.req(), "layouts/main", "dash/index", &data);

    conn.send_resp(200, &view);
}

/// Fetches a list of images matching the named tag
/// `GET /tags/{name}`
pub fn show_tags(conn: &mut plug::Conn) {
    use models::{queries, Entry};

    let tag_name = { conn.find::<MatchContext>()
        .expect("could not read route params")
        .get("name")
        .expect("could not find entry ID in route params")
        .clone()
    };

    // load entry pointers for this tag
    let results = queries::find_tag(conn, &tag_name)
        .and_then(|tag| queries::find_entries_for(conn, tag.id))
        .unwrap_or(vec![]);

    let data = EntryListView { entries: results };
    let view = views::render_into(conn.req(), "layouts/main", "dash/list", &data);
    conn.send_resp(200, &view);
}

/// Fetch the file for a given entry ID
/// `GET /show/{id}`
pub fn show_id(conn: &mut plug::Conn) {
    use models::{queries, Entry};

    let file_id: i64 = conn.find::<MatchContext>()
        .expect("could not read route params")
        .get("id")
        .expect("could not find entry ID in route params")
        .parse()
        .expect("entry ID must be a number");

    match queries::find_entry(conn, file_id) {
        Some(entry) => {
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

        None => conn.send_resp(404, "file not found"),
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


/// Extracts a file from a multipart form if the key exists & it is a file
fn extract_file(form: &mut MultipartForm, field: &str) -> Option<SavedFile> {
    match form.entries.remove(field) {
        Some(FormField::File(file)) => Some(file),
        Some(_) => { warn!("file expected, but got string"); None },
        None    => { warn!("file expected, but not present"); None },
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
