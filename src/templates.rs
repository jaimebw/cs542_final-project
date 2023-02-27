use tera::Tera;

/// Based on the example, it appears you may need to configure the templating before use to do some
/// things.
///
/// https://github.com/Keats/tera/blob/master/examples/basic/main.rs
pub fn setup_template_loader() -> tera::Result<Tera> {
    let tera = Tera::new("static/templates/**/*")?;

    Ok(tera)
}
