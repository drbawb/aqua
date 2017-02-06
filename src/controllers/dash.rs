use aqua_web::plug;
use aqua_web::mw::router::Router;

use models::{self, queries};
use views;

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
    let entries = queries::all_entries(conn);
    println!("got entries: {:?}", entries);

    // render template
    let data = Wat { derp: format!("entry => {:?}", entries) };
    let view = views::render_into(conn.req(), "layouts/main", "dash/index", &data);

    conn.send_resp(200, &view);
}

/// Fetches a list of images matching the named tag
/// `GET /tags/{schema}/{name}`
pub fn show_tags(conn: &mut plug::Conn) {
    let tag_name = Router::param::<String>(conn, "name")
        .expect("missing route param: name");

    let schema_name = Router::param::<String>(conn, "schema")
        .expect("missing route param: schema");
    
    // load entry pointers for this tag
    let results = queries::find_tag(conn, &schema_name, &tag_name)
        .and_then(|tag| queries::find_entries_for(conn, tag.id))
        .unwrap_or(vec![]);

    let data = EntryListView { entries: results };
    let view = views::render_into(conn.req(), "layouts/main", "dash/list", &data);
    conn.send_resp(200, &view);
}
