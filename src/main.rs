#[macro_use] extern crate log;
#[macro_use] extern crate mime;
#[macro_use] extern crate serde_derive;

extern crate env_logger;
extern crate handlebars;
extern crate iron;
extern crate mime_guess;
extern crate r2d2;
extern crate r2d2_postgres;
extern crate postgres;
extern crate router;
extern crate serde_json;
extern crate time;

mod util;
mod controllers;
mod models;
mod views;

use iron::{Chain, Iron};
use router::Router;
use r2d2_postgres::{TlsMode, PostgresConnectionManager};

fn main() {
    env_logger::init().expect("could not initialize console logging");

    // TODO: load these by walking directory ...
    info!("creating template registry ...");

    // TODO: set us up the database
    let config  = r2d2::Config::default();
    let manager = PostgresConnectionManager::new("postgres://drbawb@192.168.1.11/aqua_rs", TlsMode::None).unwrap();
    let pool    = r2d2::Pool::new(config, manager).unwrap();

    // TODO: set us up the chain ...
    let mut router = Router::new();
    router.get("/dash", controllers::dash::index, "dash#index");

    let mut chain = Chain::new(router);
    chain.link_before(util::template::TemplateMiddleware::new());
    chain.link_before(util::db::DbMiddleware::new(pool));
    chain.link_before(util::timer::ResponseTime);
    chain.link_after(util::timer::ResponseTime);
    chain.link_around(util::try_file::TryFile);

    Iron::new(chain)
         .http("0.0.0.0:3000")
         .expect("could not start web server");
}
