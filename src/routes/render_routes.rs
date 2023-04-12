use crate::session::Session;
use rocket::get;
use rocket_dyn_templates::{context, Template};
use rocket::request::FlashMessage;
// TODO:
//  - Add the session to the context so it can check in Tera, if the user is authenticated
//      Maybe a is_anonymous method could work
// TODO: The login checks within this file should be done via a middleware

#[get("/login")]
pub async fn login_page(session: Session<'_>,flash: Option<FlashMessage<'_>>) -> Template {
    // Template render of the login page
    if !session.is_logged_in() {
        Template::render("login", context! {flash:flash.map(FlashMessage::into_inner)})
    } else {
        Template::render("index", context! {})
        
    }
}

#[get("/signup")]
pub async fn signup_page(session: Session<'_>,flash: Option<FlashMessage<'_>>) -> Template {
    if session.user_id().is_none() {
        Template::render("register", context! {flash:flash.map(FlashMessage::into_inner)})
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
#[get("/about")]
pub async fn about_page() -> Template {
    Template::render("about", context! {})
}
