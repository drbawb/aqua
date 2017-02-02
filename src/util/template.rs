use std::sync::{Arc, RwLock};

use handlebars::Handlebars;
use iron::BeforeMiddleware;
use iron::prelude::*;
use iron::typemap;

pub struct TemplateMiddleware {
    engine: Arc<RwLock<Handlebars>>
}

impl TemplateMiddleware {
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.register_template_file("layouts/main", "./priv/templates/layouts/main.html.hbs")
            .expect("could not register layouts#main template") ;
        
        handlebars.register_template_file("dash/index", "./priv/templates/dash/index.html.hbs")
                  .expect("could not register dash#index template");

        TemplateMiddleware { engine: Arc::new(RwLock::new(handlebars)) }
    }

    pub fn refresh(&self) {
        let mut handlebars = self.engine.write().expect("could not lock template registry");
        handlebars.unregister_template("layouts/main");
        handlebars.unregister_template("dash/index");

        handlebars.register_template_file("layouts/main", "./priv/templates/layouts/main.html.hbs")
            .expect("could not register layouts#main template") ;
        
        handlebars.register_template_file("dash/index", "./priv/templates/dash/index.html.hbs")
                  .expect("could not register dash#index template");
    }
}

pub struct TemplateEngine;
impl typemap::Key for TemplateEngine { type Value = Arc<RwLock<Handlebars>>; }

impl BeforeMiddleware for TemplateMiddleware {
    fn before(&self, req: &mut Request) -> IronResult<()> {
        // insert template engine into extensions
        self.refresh(); // TODO: only reload templates in dev mode?
        req.extensions.insert::<TemplateEngine>(self.engine.clone());
        Ok(())
    }
}
