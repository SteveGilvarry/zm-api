use tera::Tera;

use crate::dto::Template;

#[derive(Clone)]
pub struct TemplateEngine {
    tera: Tera,
}

impl TemplateEngine {
    pub fn new(path: &str) -> tera::Result<Self> {
        Ok(Self {
            tera: Tera::new(path)?,
        })
    }

    pub fn render(&self, template: &Template) -> Result<String, tera::Error> {
        let (ctx, path) = template.get();
        self.tera.render(path, &ctx)
    }
}
