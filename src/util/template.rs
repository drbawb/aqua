use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use aqua_web::plug;
use glob::glob;
use handlebars::Handlebars;

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

        let glob_paths = glob("./priv/templates/**/*.hbs")
            .expect("error compiling pattern to glob templates directory")
            .filter_map(Result::ok)
            .filter(|path| path.is_file());

        for path in glob_paths {
            // this monstrosity joins the paths by "/"
            // this is a framework path, it is not used as an OS path
            let rel_dir = path.strip_prefix("priv/templates")
                .unwrap()
                .parent()
                .unwrap()
                .components()
                .map(|component| component.as_os_str())
                .map(|path_str| path_str.to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .as_slice().join("/");


            // NOTE: last component must be a file, we filter them above
            let file = path.file_stem().unwrap()
                .to_string_lossy().into_owned();
          
            // register this template under a framework path
            let framework_path = format!("{}/{}", rel_dir, file);
            templates.insert(framework_path, path.to_string_lossy().into_owned());
            info!("registered: {}/{} => {}", rel_dir, file, path.display());
        }

        let template_mw = TemplateMiddleware {
            engine: Arc::new(RwLock::new(handlebars)),
            templates: Arc::new(templates),
        };

        template_mw.refresh(); template_mw
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
