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
/*
/// This code is for creating the url_for function that
/// works inside the tera form as {{url_for()}} to redirect
use tera::Function;
use serde_json::Value;
use serde_json::from_value;
use serde_json::to_value;
use std::collections::BTreeMap;
use std::boxed::Box;
fn make_url_for(urls: BTreeMap<String, String>) -> impl Function {
    // url_for function to be used in the templates as flask
    //
    Box::new(move |args| -> Result<Value> {
        match args.get("name") {
            Some(val) => match from_value::<String>(val.clone()) {
                Ok(v) =>  Ok(to_value(urls.get(&v).unwrap()).unwrap()),
                Err(_) => Err("oops".into()),
            },
            None => Err("oops".into()),
        }
    })
}
tera::register_function("url_for",make_url_for(urls));
*/
