#[macro_use] extern crate log;
#[macro_use] extern crate mime;
#[macro_use] extern crate serde_derive;

extern crate env_logger;
extern crate handlebars;
extern crate iron;
extern crate mime_guess;
extern crate router;
extern crate serde_json;
extern crate time;

mod util;
mod controllers;
mod views;


use handlebars::Handlebars;
use iron::{Chain, Iron};
use router::Router;

fn main() {
    env_logger::init().expect("could not initialize console logging");

    // TODO: load these by walking directory ...
    info!("creating template registry ...");
    let mut handlebars = Handlebars::new();
    handlebars.register_template_file("layouts/main", "./priv/templates/layouts/main.html.hbs")
        .expect("could not register layouts#main template") ;
    handlebars.register_template_file("dash/index", "./priv/templates/dash/index.html.hbs")
              .expect("could not register dash#index template");

    // TODO: set us up the chain ...
    let mut router = Router::new();
    router.get("/dash", controllers::dash::index, "dash#index");

    let mut chain = Chain::new(router);
    chain.link_before(util::TemplateMiddleware::new(handlebars));
    chain.link_before(util::ResponseTime);
    chain.link_after(util::ResponseTime);
    chain.link_around(util::TryFile);

    Iron::new(chain)
         .http("0.0.0.0:3000")
         .expect("could not start web server");
}
