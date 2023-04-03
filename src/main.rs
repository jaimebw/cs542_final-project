use crate::env::{setup_dotenv, var};
use log::{error, warn, LevelFilter};
use rocket::{Build, Rocket};
use rocket_dyn_templates::Template;

use crate::templates::{setup_template_loader, TemplateUrlLoader};
use error::MixedResult as Result;

mod database;
mod env;
mod error;
mod forms;
mod routes;
mod session;
mod templates;
mod scraper;

type AnyResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    setup_logging();
    setup_dotenv();

    // Begin tokio async runtime
    let err: AnyResult<()> = tokio::runtime::Builder::new_multi_thread()
        .thread_name("rocket-worker-thread")
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let _ = build_rocket().await?.launch().await?;
            Ok(())
        });

    match err {
        Ok(_) => warn!("Program exited early without error"),
        Err(e) => error!("Program exited early with error {}", e),
    }
}

async fn build_rocket() -> AnyResult<Rocket<Build>> {
    // Create database pool
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&var("DATABASE_URL"))
        .await?;

    // Create and launch rocket server and initialize managed resources
    let app = routes::build_app();
    let url_loader = TemplateUrlLoader::from(&app);

    // Create template loader
    let templates = Template::try_custom(move |builder| {
        // Pass template engine to setup function
        setup_template_loader(&mut builder.tera, url_loader.to_owned())?;
        Ok(())
    });

    Ok(app.attach(templates).manage(pool))
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
        .filter_module("sqlx", LevelFilter::Warn)
        // .filter_module("_", LevelFilter::Warn)
        .init();
}
