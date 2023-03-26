//! TODO: The login checks within this file should be done via a middleware
use crate::session::Session;
use rocket::get;
use rocket_dyn_templates::{context, Template};

#[get("/login")]
pub async fn login_page(session: Session<'_>) -> Template {
    // Template render of the login page
    // Added condition if a user is currently logged in, it cannot go to this page again
    if !session.is_logged_in() {
        Template::render("login", context! {})
    } else {
        Template::render("index", context! {})
    }
}

#[get("/signup")]
pub async fn signup_page(session: Session<'_>) -> Template {
    if session.user_id().is_none() {
        Template::render("register", context! {})
    } else {
        Template::render("index", context! {})
    }
}

#[get("/index")]
pub async fn index_page(session: Session<'_>) -> Template {
    // Template render of the index
    if session.user_id().is_none() {
        Template::render("login", context! {})
    } else {
        Template::render("index", context! {})
    }
}
