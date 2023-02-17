use rocket::fs::FileServer;
use rocket::{get, routes, Build, Rocket};

pub mod user;

/// Setup all of the routes used by the app
pub fn build_app() -> Rocket<Build> {
    rocket::build()
        .mount("/static", FileServer::from("./static"))
        .mount("/index", routes![index_page])
        .mount("/user", routes![user::login_page, user::user_homepage])
        .mount(
            "/api/user",
            routes![user::login, user::sign_up, user::logout],
        )
}

#[get("/")]
pub fn index_page() -> &'static str {
    "TODO: Load the homepage with some trending products"
}
