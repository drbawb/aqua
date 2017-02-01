use util::TemplateEngine;

use std::sync::Arc;

use handlebars::Handlebars;
use iron::request::Request;
use serde_json::value::ToJson;

#[derive(Serialize)]
struct Layout { inner: String }

pub fn req_engine(req: &Request) -> Arc<Handlebars> {
    req.extensions.get::<TemplateEngine>()
        .expect("template engine requested, but not available!")
        .clone()
}

pub fn render_into<T>(registry: &Handlebars, layout: &str, template: &str, data: &T) -> String
where T: ToJson {
    let inner_html = registry.render(template, data)
                             .expect("TODO: error rendering template");

    registry.render(layout, &Layout { inner: inner_html })
        .expect("TODO: error rendering layout")

}
