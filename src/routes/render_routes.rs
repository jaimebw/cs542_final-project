use crate::session::Session;
use log::{error, info};
use rocket::get;
use rocket_dyn_templates::{context, Template};
use rocket::request::FlashMessage;
use sqlx:: Sqlite;
use sqlx::FromRow;
use crate::database::Connection;
use serde::{Serialize,Deserialize};
use rocket::serde::json::json;

// TODO:
//  - Add the session to the context so it can check in Tera, if the user is authenticated
//      Maybe a is_anonymous method could work
// TODO: The login checks within this file should be done via a middleware
#[derive(FromRow,Serialize)]
struct Product { ASIN: String , conditions:String}


#[get("/login")]
pub async fn login_page(session: Session<'_>,mut database: Connection<Sqlite>,
                        flash: Option<FlashMessage<'_>>) -> crate::Result<Template> {
    // Template render of the login page
    if !session.is_logged_in() {
        Ok(Template::render("login", context! {flash:flash.map(FlashMessage::into_inner)}))
    } 
    else {

        Ok(Template::render("index", context! {}) ) 
        }
}

        

#[get("/signup")]
pub async fn signup_page(session: Session<'_>,flash: Option<FlashMessage<'_>>) -> crate::Result<Template> {
    // I need to add here the products_json
    if session.user_id().is_none() {
        Ok(Template::render("register", context! {flash:flash.map(FlashMessage::into_inner)}))
    } else {
        Ok(Template::render("index", context! {}))
    }
}

#[get("/index")]
pub async fn index_page(session: Session<'_>,mut database: Connection<Sqlite>) -> crate::Result<Template> {
    // Template render of the index
    if session.user_id().is_none() {
        Ok(Template::render("login", context! {}))
    } 
    else {
        // Add more info to the query, we need the name of the product
        let user_products=
            sqlx::query_as::<_, Product>("SELECT ASIN, conditions FROM Subscribes_To WHERE sid = ?")
            .bind(&session.user_id())
            .fetch_all(&mut *database)
            .await?;

        Ok(Template::render("index", context! {
            products: &user_products
        }))
    }
}

#[get("/about")]
pub async fn about_page() -> crate::Result<Template> {
    Ok(Template::render("about", context! {}))
}
