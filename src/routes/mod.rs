use crate::session::Session;
use rocket::fs::FileServer;
use rocket::{get, routes, Build, Rocket, State};
use tera::{Context, Tera};

pub mod user;

/// Setup all of the routes used by the app
pub fn build_app() -> Rocket<Build> {
    rocket::build()
        .mount("/static", FileServer::from("./static"))
        .mount("/index", routes![index_page])
        .mount("/user", routes![user::login_page, user::user_homepage])
        .mount("/template", routes![template_example])
        .mount(
            "/api/user",
            routes![user::login, user::sign_up, user::logout],
        )
}

#[get("/")]
pub fn index_page() -> &'static str {
    "TODO: Load the homepage with some trending products"
}

/// A basic example using the tera struct we set up in templates.rs when handling a route.
#[get("/")]
pub fn template_example(session: Session<'_>, tera: &State<Tera>) -> crate::Result<String> {
    let mut context = Context::new();

    if let Some(current_user) = session.user_id() {
        context.insert("user_id", &format!("{:?}", current_user));
    }

    Ok(tera.render("basic.html", &context)?)
}
