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
struct Product  {ASIN: String , Price:f32, datetime:String,name:String }


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
pub async fn index_page(session: Session<'_>,mut database: Connection<Sqlite>,
                        flash: Option<FlashMessage<'_>>) -> crate::Result<Template> {
    // Template render of the index
    if session.user_id().is_none() {
        Ok(Template::render("login", context! {}))
    } 
    else {
        // Add more info to the query, we need the name of the product
        let user_products=
            sqlx::query_as::<_, Product>("
                WITH Latest_Listings AS (
                SELECT
                    ASIN,
                    MAX(datetime) AS latest_datetime
                FROM
                    Has_Listing_collected
                WHERE
                    ASIN IN (
                        SELECT
                            ASIN
                        FROM
                            Subscribes_To
                        WHERE
                            sid = ?)
                GROUP BY
                    ASIN
            )
            SELECT
                hlc.ASIN,
                hlc.Price,
                hlc.datetime,
                spm.name
            FROM
                Has_Listing_collected hlc
            JOIN
                Latest_Listings ll ON hlc.ASIN = ll.ASIN AND hlc.datetime = ll.latest_datetime
            JOIN
                Product_variant_Sold pvs ON hlc.ASIN = pvs.ASIN
            JOIN
                Sold_Product_Manufactured spm ON pvs.PID = spm.PID
            ORDER BY
                hlc.ASIN;")
                        .bind(&session.user_id())
                        .fetch_all(&mut *database)
                        .await?;
        info!("Query works!");

        Ok(Template::render("index", context! {
            products: &user_products,
            flash: flash.map(FlashMessage::into_inner)
        }))
    }
}

#[get("/about")]
pub async fn about_page() -> crate::Result<Template> {
    Ok(Template::render("about", context! {}))
}
