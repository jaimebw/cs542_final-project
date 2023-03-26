mod url_for;

pub use crate::templates::url_for::TemplateUrlLoader;
use tera::Tera;

/// Based on the example, it appears you may need to configure the templating before use to do some
/// things.
///
/// https://github.com/Keats/tera/blob/master/examples/basic/main.rs
pub fn setup_template_loader(tera: &mut Tera, url_loader: TemplateUrlLoader) -> tera::Result<()> {
    tera.register_function("url_for", url_loader);

    Ok(())
}
