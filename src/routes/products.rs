use crate::database::Connection;
use rocket_dyn_templates::{context};
use crate::error::Error;
use crate::scraper::{extract_asin, AmazonApi};
use crate::session::UserId;
use rocket::{get, State};
use rocket_dyn_templates::Template;
use sqlx::Sqlite;
use rocket::serde::json::json;
use log::info;

#[get("/add?<url>")]
pub async fn add_product(
    user: UserId,
    mut database: Connection<Sqlite>,
    amazon_api: &State<AmazonApi>,
    url: &str,
) -> crate::Result<Template>{
    // This method should results in adding the product information to the database
    // There should be no template as response
    //
    info!("The requested URL is \n {}",&url);
    let asin = match extract_asin(url) {
        Some(asin) => asin.to_ascii_uppercase(),
        None => return Err(Error::from("Requested link must be an amazon URL")),
    };

    /*
use std::time::Duration;
use tokio::time::timeout;
    info!("ASIN: {}",&asin);
    let product = match timeout(Duration::from_secs(5), amazon_api.get_product_info(&asin)).await {
        Ok(result) => result?,
        Err(_) => return Err("Timeout waiting for product information".into()),
    };
    */
    let product = match amazon_api.get_product_info(&asin).await? {
        Some(product) => {
            info!("{}",&product.name);
            product},
        None => {
            info!("No products found");
            return Err(Error::from("Unable to find the selected product"))},
    };
    

    // TODO: Add product to the database if it is not already there
    // TODO: Add product to user's tracked product list
    // TODO: Return a template for the reloaded page

    // Get the user's product list with the newly updated entry
    //tracked_product_list(user, database).await
    Ok(Template::render("index", context!{
        product:json!(product)

    }))
    
}

#[get("/remove?<asin>")]
pub async fn remove_product(
    user: UserId,
    mut database: Connection<Sqlite>,
    asin: &str,
) -> crate::Result<Template> {
    // TODO: Remove the product from the current user's tracked product list

    tracked_product_list(user, database).await
}

#[get("/update?<asin>")]
pub async fn update_now(
    user: UserId,
    mut database: Connection<Sqlite>,
    amazon_api: &State<AmazonApi>,
    asin: &str,
) -> crate::Result<Template> {
    // TODO: Verify that asin is being tracked by the current user

    let offers = amazon_api.get_offers_for_asin(&asin).await?;

    // TODO: Add the new offers to database

    // Return the product page with the newly updated data
    product_info(database, asin).await
}

#[get("/list")]
pub async fn tracked_product_list(
    user: UserId,
    mut database: Connection<Sqlite>,
) -> crate::Result<Template> {
    // TODO: Get all products tracked by this user from the database and insert their names
    // TODO: Return template with all of that user's templates filled in
    todo!()
}

#[get("/info?<asin>")]
pub async fn product_info(mut database: Connection<Sqlite>, asin: &str) -> crate::Result<Template> {
    // TODO: Fill in and return a template showing the price history of the requested product
    todo!()
}
