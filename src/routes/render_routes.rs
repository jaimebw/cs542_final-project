use crate::database::Connection;
use crate::forms::UserCredentials;
use rocket_dyn_templates::{Template, tera::Tera, context};
use crate::error::Error;
use crate::session::{Session, UserId};
use log::info;
use rocket::response::Redirect;
use rocket::{get, post, uri};
use sqlx::types::Uuid;
use sqlx::Sqlite;
use rocket::form::Form;

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