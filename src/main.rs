#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate log;
#[macro_use] extern crate mime;
#[macro_use] extern crate serde_derive;

extern crate aqua_web;
extern crate conduit;
extern crate conduit_hyper;
extern crate crypto;
extern crate dotenv;
extern crate env_logger;
extern crate handlebars;
extern crate mime_guess;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate serde_json;
extern crate time;

mod controllers;
mod models;
mod schema;
mod util;
mod views;

use aqua_web::mw::Router;
use aqua_web::plug;
use conduit::Method;
use conduit_hyper::Server;
use dotenv::dotenv;


fn main() {
    dotenv().expect("must provide .env file, see README (TODO: haha jk)");
    env_logger::init().expect("could not initialize console logging");

    // TODO: load these by walking directory ...
    info!("creating template registry ...");

    // TODO: set up some basic middlewre
    let mut router = Router::new();
    router.add_route(Method::Get, "/dash", controllers::dash::index);
    router.add_route(Method::Post, "/entries/upload", controllers::dash::submit);

    let mut pipeline = plug::Pipeline::new();
    pipeline.register(util::timer::plug);
    pipeline.register(util::db::DbMiddleware::new());
    pipeline.register(util::template::TemplateMiddleware::new());
    pipeline.register(util::try_file::TryFileMiddleware);
    pipeline.register(router);

    Server::http("0.0.0.0:3000")
        .expect("could not start http server")
        .handle(pipeline)
        .expect("could not attach handler");
}
