use crate::session::Session;
use rocket::{get, routes, Build, Rocket};
use rocket_dyn_templates::{context, Template};

pub mod user;
pub mod render_routes;

#[cfg(test)]
mod tests;

/// Setup all of the routes used by the app
pub fn build_app() -> Rocket<Build> {
    rocket::build()
        .mount("/", routes![
               user::index,
               user::register,
               user::logout,
               user::login])
        .mount("/", routes![
               render_routes::index_page,
               render_routes::login_page,
        render_routes::signup_page])
}



// A basic example using the tera struct we set up in templates.rs when handling a route.
/*
#[get("/")]
pub fn template_example(session: Session<'_>) -> Template {
    let mut user_id = None;

    if let Some(current_user) = session.user_id() {
        user_id = Some(format!("{:?}", current_user));
    }

    Template::render("basic.html", context! { user_id })
}
*/
