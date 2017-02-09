use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use aqua_web::plug;
use handlebars::Handlebars;
use walkdir::WalkDir;

static TEMPLATE_ROOT: &'static str = "./priv/templates";

/// The extension registry type of the templating engine
pub type TemplateEngine = Arc<RwLock<Handlebars>>;

pub struct TemplateMiddleware {
    engine: Arc<RwLock<Handlebars>>,
    templates: Arc<HashMap<String, String>>,
}

impl TemplateMiddleware {
    pub fn new() -> Self {
        let handlebars = Handlebars::new();
        let mut templates = HashMap::new();

        for entry in WalkDir::new(TEMPLATE_ROOT) {
            let entry = entry.expect("error reading template dir");

            // skip directory entries
            let file_ty = entry.file_type();
            if file_ty.is_dir() { continue }

            // skip entries that aren't .hbs files
            let is_template = match entry.path().extension() {
                Some(file_ext) => file_ext == "hbs",
                None => false,
            };

            if !is_template { continue }

            // take off the root prefix to generate a framework name
            // (which is really just the path relative to the *template root* rather
            // than the framework root.
            let entry_path = entry.path()
                .strip_prefix(TEMPLATE_ROOT)
                .unwrap();

            // create a normalized framework path (e.g: "controller/index")
            let entry_dir = entry_path.parent().unwrap();
            let entry_file = entry_path.file_stem().unwrap().to_string_lossy();
            let template_path = entry_dir.iter()
                .map(|derp| derp.to_str().unwrap())
                .collect::<Vec<_>>()
                .join("/");

            let template_real_path = entry.path().to_string_lossy().into_owned();
            let template_framework_name = format!("{}/{}", template_path, entry_file);
           
            // store it so they can be loaded on demand
            info!("found template: {} @ {}", template_framework_name, template_real_path);
            templates.insert(template_framework_name, template_real_path);
        }

        let templates_mw = TemplateMiddleware {
            engine: Arc::new(RwLock::new(handlebars)),
            templates: Arc::new(templates),
        };

        // load them up at startup once ...
        templates_mw.refresh(); templates_mw
    }

    pub fn refresh(&self) {
        let mut handlebars = self.engine.write().expect("could not lock template registry");

        for (template_mount, template_path) in &*self.templates {
            handlebars.unregister_template(template_mount);
            handlebars.register_template_file(template_mount, template_path)
                .expect("could not register layouts#main template") ;
        }
    }

}

impl plug::Plug for TemplateMiddleware {
    fn call(&self, conn: &mut plug::Conn) {
        // insert template engine into extensions
        self.refresh(); // TODO: only reload templates in dev mode?
        conn.req_mut().mut_extensions().insert::<TemplateEngine>(self.engine.clone());
    }
}
