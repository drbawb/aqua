#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
#[macro_use] extern crate log;
#[macro_use] extern crate mime;
#[macro_use] extern crate serde_derive;

extern crate crypto;
extern crate dotenv;
extern crate env_logger;
extern crate handlebars;
extern crate iron;
extern crate mime_guess;
extern crate params;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate router;
extern crate serde_json;
extern crate time;

mod controllers;
mod models;
mod schema;
mod util;
mod views;

use dotenv::dotenv;
use iron::{Chain, Iron};
use router::Router;

fn main() {
    dotenv().expect("must provide .env file, see README (TODO: haha jk)");
    env_logger::init().expect("could not initialize console logging");

    // TODO: load these by walking directory ...
    info!("creating template registry ...");

    // TODO: set us up the database


    // TODO: set us up the chain ...
    let mut router = Router::new();
    router.get("/dash", controllers::dash::index, "dash#index");
    router.post("/entries/upload", controllers::dash::submit, "dash#submit");

    let mut chain = Chain::new(router);
    chain.link_before(util::template::TemplateMiddleware::new());
    chain.link_before(util::db::DbMiddleware::new());
    chain.link_before(util::timer::ResponseTime);
    chain.link_after(util::timer::ResponseTime);
    chain.link_around(util::try_file::TryFile);

    Iron::new(chain)
         .http("0.0.0.0:3000")
         .expect("could not start web server");
}
