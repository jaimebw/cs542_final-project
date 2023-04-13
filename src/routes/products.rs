use crate::database::Connection;
use rocket::response::{Flash, Redirect};
use crate::error::Error;
use crate::scraper::{extract_asin, AmazonApi};
use crate::session::UserId;
use rocket::{get, State};
use rocket_dyn_templates::Template;
use sqlx::Sqlite;
use log::info;




#[get("/add?<url>")]
pub async fn add_product(
    mut database: Connection<Sqlite>,
    amazon_api: &State<AmazonApi>,
    url: &str,
) -> crate::Result<Flash<Redirect>>{
    // This method should results in adding the product information to the database
    // There should be no template as response
    //
    info!("The requested URL is \n {}",&url);

    // Logic of the request:
    //  1. Get the ASIN number
    //      1.1 If error -> Flash not found product 
    //  2. Get product info with ASIN
    //      2.1 If error -> Flash not found product
    //      2.2 No SQL INSERT operation
    //  3. SQL Insert into db 
    let mut flash = Flash::success(Redirect::to("/index"), "Added product!");

    let asin = match extract_asin(url) {
        Some(asin) => asin.to_ascii_uppercase(),
        None => {
            flash = Flash::error(Redirect::to("/index"), "Product not found");
            return Err(Error::FlashError(flash));
        }
    };

    let product = match amazon_api.get_product_info(&asin).await? {
        Some(product) => {
            info!("{}",&product.name);
            product
        },
        None => {
            info!("No products found");
            flash = Flash::error(Redirect::to("/index"), "Product not found");
            return Err(Error::FlashError(flash));
        }
    };

    // TODO: INSERT Query here
    // TODO: see how the product variable transform to a json to introduce it inside the INSERT
    // operation

    Ok(Flash::success(Redirect::to("/index"),"Added new product" ))
       
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
