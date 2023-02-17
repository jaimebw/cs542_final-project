use rocket::get;

pub mod user;

#[get("/")]
pub fn index_page() -> &'static str {
    "TODO: Load the homepage with some trending products"
}
