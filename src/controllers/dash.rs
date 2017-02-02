use controllers::prelude::*;
use models::Entry;
use views;

#[derive(Serialize)]
struct Wat {
    derp: String,
}

/// Does the thing, wins the points ...
pub fn index(req: &mut Request) -> IronResult<Response> {
    // load model
    let entry = Entry::find_by_hash(req, "0e4c9effdba2549c098a858f8cfa76cc96bf3a1ed47a4dbdce435e5fa4dd2078");
    let data   = Wat { derp: format!("entry => {:?}", entry) };

    // render template
    let view   = views::render_into(req, "layouts/main", "dash/index", &data);
    Ok(Response::with((status::Ok, view))
                .set(mime!(Text/Html)))
}
