use controllers::prelude::*;
use views;

#[derive(Serialize)]
struct Wat {
    derp: String,
}

/// Does the thing, wins the points ...
pub fn index(req: &mut Request) -> IronResult<Response> {
    // render template
    let data   = Wat { derp: "oh my".to_string() };
    let engine = views::req_engine(req);
    let view   = views::render_into(&engine, "layouts/main", "dash/index", &data);


    Ok(Response::with((status::Ok, view))
                .set(mime!(Text/Html)))
}
