use crate::database::Connection;
use crate::error::Error;
use crate::scraper::{extract_asin, AmazonApi};
use crate::session::UserId;
use rocket::{get, State};
use rocket_dyn_templates::Template;
use sqlx::Sqlite;

#[get("/add?<url>")]
pub async fn add_product(
    user: UserId,
    mut database: Connection<Sqlite>,
    amazon_api: &State<AmazonApi>,
    url: &str,
) -> crate::Result<Template> {
    let asin = match extract_asin(url) {
        Some(asin) => asin.to_ascii_uppercase(),
        None => return Err(Error::from("Requested link must be an amazon URL")),
    };

    let product = match amazon_api.get_product_info(&asin).await? {
        Some(product) => product,
        None => return Err(Error::from("Unable to find the selected product")),
    };

    // TODO: Add product to the database if it is not already there
    // TODO: Add product to user's tracked product list
    // TODO: Return a template for the reloaded page

    // Get the user's product list with the newly updated entry
    tracked_product_list(user, database).await
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
