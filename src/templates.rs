use tera::Tera;

/// Based on the example, it appears you may need to configure the templating before use to do some
/// things.
///
/// https://github.com/Keats/tera/blob/master/examples/basic/main.rs
pub fn setup_template_loader(tera: &mut Tera) -> tera::Result<()> {
    // The default initializer does not inspect the subdirectories of templates, so we just restart
    // the template engine with the corrected file glob.
    *tera = Tera::new("templates/**/*")?;

    Ok(())
}
