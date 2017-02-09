use util::template::TemplateEngine;

use conduit::Request;
use serde_json::value::ToJson;

#[derive(Serialize)]
struct Layout { inner: String }

pub fn render<T>(req: &Request, template: &str, data: &T) -> String
where T: ToJson {
    let engine = req.extensions().find::<TemplateEngine>()
        .expect("template engine requested, but not available!")
        .clone();

    let registry = engine.read().expect("could not lock the template engine");
    registry.render(template, data).expect("TODO: error rendering template")
}

pub fn render_into<T>(req: &Request, layout: &str, template: &str, data: &T) -> String
where T: ToJson {
    let engine = req.extensions().find::<TemplateEngine>()
        .expect("template engine requested, but not available!")
        .clone();

    let registry = engine.read().expect("could not lock the template engine");

    let inner_html = registry.render(template, data)
                             .expect("TODO: error rendering template");

    registry.render(layout, &Layout { inner: inner_html })
        .expect("TODO: error rendering layout")

}
