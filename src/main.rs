extern crate aqua;
extern crate aqua_web;
extern crate conduit_hyper;
extern crate dotenv;
extern crate env_logger;

use aqua::{controllers, util};
use aqua_web::{mw, plug};
use conduit_hyper::Server;
use dotenv::dotenv;

fn main() {
    // load configuration from .env
    dotenv().expect("must provide .env file, see README (TODO: haha jk)");
    env_logger::init().expect("could not initialize console logging");

    // these are application extensions which our controllers expect to be present
    let extensions = plug::Pipeline::new()
        .then(util::db::DbMiddleware::new())
        .then(util::template::TemplateMiddleware::new());

    // the main entry point into our application
    let router = mw::Router::new()
        .get("/dash",                 controllers::dash::index)
        .get("/tags/{schema}/{name}", controllers::dash::show_tags)
        .get("/entries/{id}",         controllers::entries::show)
        .get("/entries/{id}/thumb",   controllers::entries::show_thumb)
        .get("/entries/{id}/tags",    controllers::entries::show_entry_tags)
        .post("/entries/upload",      controllers::entries::submit);

    // the endpoint provides basic HTTP massaging before our router is invoked
    // with the current request data ...
    let endpoint = plug::Pipeline::new()
        .then(util::timer::plug)
        .then(util::try_file::TryFileMiddleware)
        .then(mw::MultipartParser)
        .then(extensions)
        .then(router);

    Server::http("0.0.0.0:3000")
        .expect("could not start http server")
        .handle(endpoint)
        .expect("could not attach handler");
}
