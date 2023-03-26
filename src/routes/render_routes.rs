use rocket_dyn_templates::{Template, tera::Tera, context};
use crate::session::{Session, UserId};
use log::info;
use rocket::{get, post};

// TODO:
//  - Add the session to the context so it can check in Tera, if the user is authenticated
//      Maybe a is_anonymous method could work

#[get("/login")]
pub async fn login_page(session: Session<'_>) -> Template { 
    // Template reder of the login logi page 
    // Added condition if a user is currently logged in, it cannot go to this page again
    if session.user_id().is_none(){
        Template::render("login", context!{})
    }
    else{
        Template::render("index",context! {})
    }
}

#[get("/signup")]
pub async fn signup_page(session: Session<'_>) ->Template{
    if session.user_id().is_none(){
        Template::render("register",context! {})
    }
    else{
        Template::render("index",context! {})
    }
}

#[get("/index")]
pub async fn index_page(session: Session<'_>) -> Template{
    // Template render of the index
    if session.user_id().is_none(){
    
        Template::render("login", context! {})
    }
    else{
        Template::render("index", context! {})
    }
}
#[get("/about")]
pub async fn about_page() -> Template{
    Template::render("about",context! {})
}
