use crate::env::{setup_dotenv, var};
use log::{error, warn, LevelFilter};
use rocket::fs::FileServer;
use rocket::routes;

mod database;
mod env;
mod routes;
mod session;

fn main() {
    setup_logging();
    setup_dotenv();

    // Begin tokio async runtime
    let err = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(begin_async());

    match err {
        Ok(_) => warn!("Program exited early without error"),
        Err(e) => error!("Program exited early with error {}", e),
    }
}

async fn begin_async() -> anyhow::Result<()> {
    // Create database pool
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&var("DATABASE_URL"))
        .await?;

    // Create rocket server and initialize the routes to be used
    let _ = rocket::build()
        .manage(pool)
        .mount("/static", FileServer::from("./static"))
        .mount("/index", routes![routes::index_page])
        .mount(
            "/user",
            routes![routes::user::login_page, routes::user::user_homepage],
        )
        .mount(
            "/api/user",
            routes![
                routes::user::login,
                routes::user::sign_up,
                routes::user::logout
            ],
        )
        .launch()
        .await?;

    Ok(())
}

fn setup_logging() {
    pretty_env_logger::formatted_builder()
        .format_timestamp(None)
        .filter_level(LevelFilter::Debug)
        .filter_module("selectors", LevelFilter::Info)
        .filter_module("html5ever", LevelFilter::Warn)
        .filter_module("hyper", LevelFilter::Warn)
        .filter_module("reqwest", LevelFilter::Warn)
        .filter_module("cookie_store", LevelFilter::Warn)
        .init();
}
