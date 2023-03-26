use crate::session::Session;
use rocket::{get, routes, Build, Rocket};
use rocket_dyn_templates::{context, Template};

pub mod user;
pub mod render_routes;

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
               render_routes::signup_page,
                render_routes::about_page])
}

