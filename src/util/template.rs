use std::sync::{Arc, RwLock};

use aqua_web::mw::{Outcome, Segment, Wrapper};
use conduit::Request;
use handlebars::Handlebars;

/// The extension registry type of the templating engine
pub type TemplateEngine = Arc<RwLock<Handlebars>>;

pub struct TemplateMiddleware;

impl Wrapper for TemplateMiddleware {
    fn around(self, handler: Box<Segment>) -> Box<Segment> {
        Box::new(TemplateHandler::new(handler))
    }
}

struct TemplateHandler {
    next: Box<Segment>,
    engine: Arc<RwLock<Handlebars>>
}

impl TemplateHandler {
    pub fn new(next: Box<Segment>) -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.register_template_file("layouts/main", "./priv/templates/layouts/main.html.hbs")
            .expect("could not register layouts#main template") ;
        
        handlebars.register_template_file("dash/index", "./priv/templates/dash/index.html.hbs")
                  .expect("could not register dash#index template");

        TemplateHandler { next: next, engine: Arc::new(RwLock::new(handlebars)) }
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



impl Segment for TemplateHandler {
    fn invoke(&self, req: &mut Request) -> Outcome {
        // insert template engine into extensions
        self.refresh(); // TODO: only reload templates in dev mode?
        req.mut_extensions().insert::<TemplateEngine>(self.engine.clone());
        self.next.invoke(req)
    }
}
