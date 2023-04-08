use rocket::{routes, Build, Rocket,get};


pub mod render_routes;
pub mod user;

mod products;
#[cfg(test)]
mod tests;

/// Setup all of the routes used by the app
pub fn build_app() -> Rocket<Build> {
    rocket::build()
        .mount("/",
               routes![cart_json])
        .mount(
            "/",
            routes![user::index, user::register, user::logout, user::login],
        )
        .mount(
            "/",
            routes![
                render_routes::index_page,
                render_routes::login_page,
                render_routes::signup_page,
                render_routes::about_page
            ],
        )

        .mount(
            "/product",
            routes![
                products::add_product,
                products::remove_product,
                products::update_now,
                products::tracked_product_list,
                products::product_info,
            ],
        )

}

#[get("/cart.json")]
async fn cart_json() -> &'static str {
    "This is the response for the /cart.json endpoint"
}

