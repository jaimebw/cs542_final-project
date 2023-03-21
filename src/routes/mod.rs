use crate::session::Session;
use rocket::{get, routes, Build, Rocket};
use rocket_dyn_templates::{context, Template};

pub mod user;

/// Setup all of the routes used by the app
pub fn build_app() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![user::index])
        .mount("/", routes![user::index_page])
        .mount("/", routes![user::login_page])
}


#[get("/")]
pub fn index(session: Session<'_>) -> Template{

    if session.user_id().is_none(){
        Template::render("login.html",context! {})
    }

    else{
        Template::render("index.html",context! {})
    }
}


/// A basic example using the tera struct we set up in templates.rs when handling a route.
#[get("/")]
pub fn template_example(session: Session<'_>) -> Template {
    let mut user_id = None;

    if let Some(current_user) = session.user_id() {
        user_id = Some(format!("{:?}", current_user));
    }

    Template::render("basic.html", context! { user_id })
}
