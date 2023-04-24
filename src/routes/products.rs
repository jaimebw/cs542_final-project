use crate::database::Connection;
use crate::session::Session;
use rocket::http::RawStr;
use rocket::response::{Flash, Redirect};
use crate::error::Error;
use crate::scraper::{extract_asin, AmazonApi};
use crate::session::UserId;
use rocket::{get, State};
use rocket_dyn_templates::{context, Template};
use sqlx::Sqlite;
use log::info;
use sqlx::FromRow;
use serde::Serialize;
use uuid::Uuid;
use crate::scraper::product::DepartmentHierarchy;

#[derive(FromRow,Serialize)]
struct ProductStory {Price: f32, datetime:String }
impl ProductStory {
    fn to_vectors(&self) -> (Vec<String>, Vec<f32>) {
        let mut timestamps = Vec::new();
        let mut prices = Vec::new();

        timestamps.push(self.datetime.clone());
        prices.push(self.Price);

        (timestamps, prices)
    }
}


#[get("/add?<url>")]
pub async fn add_product(
    user_id: UserId,
    mut database: Connection<Sqlite>,
    amazon_api: &State<AmazonApi>,
    url: &str,
) -> crate::Result<Flash<Redirect>> {
    // This method should results in adding the product information to the database
    // There should be no template as response
    info!("The requested URL is \n {}",&url);

    let asin = match extract_asin(url) {
        Some(asin) => asin.to_ascii_uppercase(),
        None => return Ok(Flash::error(Redirect::to("/index"), "URL must be a valid Amazon product URL")),
    };

    let product = match amazon_api.get_product_info(&asin).await? {
        Some(product) => product,
        None => {
            let flash_error = Flash::error(Redirect::to("/index"), "Product not found");
            return Err(Error::from(flash_error));
        }
    };


    let product_id = match database.product_exists(&product.asin).await? {
        Some(id) => id,
        None =>  database.add_product(&product).await?,
    };

    database.track_product(user_id, product_id).await?;

    Ok(Flash::success(Redirect::to("/index"),"Added new product" ))
}

#[get("/historic?<asin>")]
pub async fn historic(
    mut database: Connection<Sqlite>,
    asin: &str,
) -> crate::Result<Template> {
    let product_historic = 
        sqlx::query_as::<_,ProductStory>("SELECT Price, datetime FROM Has_Listing_collected WHERE ASIN = ? ")
        .bind(asin)
        .fetch_all(&mut *database)
        .await?;

    let mut timestamps = Vec::new();
    let mut prices = Vec::new();

    for story in product_historic.iter() {
        let (story_timestamps, story_prices) = story.to_vectors();
        timestamps.extend(story_timestamps);
        prices.extend(story_prices);
    }
    let max_price = prices.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b)); 
    let min_price = prices.iter().fold(f32::INFINITY, |a, &b| a.min(b)); 
    Ok(Template::render("historic",context! {
        max_price : &max_price,
        min_price: &min_price,
       prices: &prices,
       timestamps: &timestamps
    }))

}
    


#[get("/remove?<asin>")]
pub async fn remove_product(
    mut database: Connection<Sqlite>,
    asin: &str,
) -> crate::Result<Flash<Redirect>> {
    // todo: remove the product from the current user's tracked product list
    sqlx::query("DELETE from Product_variant_sold WHERE asin = ?")
        .bind(asin)
        .execute(&mut *database)
        .await?;
    info!("try to remove");
    
    Ok(Flash::success(Redirect::to("/index"),"deleted new product" ))
}

#[get("/update?<asin>")]
pub async fn update_now(
    user: UserId,
    mut database: Connection<Sqlite>,
    amazon_api: &State<AmazonApi>,
    asin: &str,
) -> crate::Result<Template> {
    // TODO: Verify that asin is being tracked by the current user
    let product = match amazon_api.get_product_info(&asin).await? {
        Some(product) => product,
        None => {
            let flash_error = Flash::error(Redirect::to("/index"), "Product not found");
            return Err(Error::from(flash_error));
        }
    };

    if database.product_exists(&product.asin).await?.is_none() {
        return Err(Error::from("Product must be added before it can be updated"))
    };

    let department = database.get_or_add_department(&product.department).await?;
    let manufacturer = database.get_or_add_manufacturer(&product.manufacturer).await?;

    sqlx::query("
    UPDATE Sold_Product_Manufactured
        SET name = ?, DepID = ?, ManuID = ?
        WHERE PID IN (SELECT PID FROM Product_variant_Sold WHERE ASIN = ?);
    ")
        .bind(&product.name)
        .bind(department)
        .bind(manufacturer)
        .bind(asin)
        .execute(&mut *database)
        .await?;

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
