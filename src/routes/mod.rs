use crate::session::Session;
use rocket::fs::FileServer;
use rocket::{get, routes, Build, Rocket};
use rocket_dyn_templates::{context, Template};

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

//#[get("/")]
//pub fn index_page() -> &'static str {
//    "TODO: Load the homepage with some trending products"
//}

#[get("/")]
pub fn index_page() -> Template{
    Template::render("index.html",context! {})
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
