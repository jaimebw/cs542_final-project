use crate::database::Connection;
use crate::forms::UserCredentials;
use rocket_dyn_templates::{Template, tera::Tera, context};
use crate::error::Error;
use crate::session::{Session, UserId};
use log::info;
use rocket::response::Redirect;
use rocket::{get, post, uri};
use sqlx::types::Uuid;
use sqlx::Sqlite;
use rocket::form::Form;


#[get("/")]
pub async fn index(session: Session<'_>,)->crate::Result<Redirect> {
    if session.user_id().is_none(){
    
        Ok(Redirect::to(uri!(login_page)))
    }
    else{
        Ok(Redirect::to(uri!(index_page)))
    }
}

#[get("/index")]
pub async fn index_page(session: Session<'_>) -> Template{
    Template::render("index", context! {})
}

#[get("/login")]
pub async fn login_page(session: Session<'_>) -> Template { 
    // Route to the login page.
    if session.user_id().is_none(){
        Template::render("login", context!{})
    }
    else{
        Template::render("index",context! {})
    }

}


#[post("/login",data = "<credentials>")]
pub async fn login(session: Session<'_>,mut database: Connection<Sqlite>,
                   credentials: Form<UserCredentials<'_>>) -> crate::Result<Redirect> {

    if !credentials.is_valid_email() {
        return Err(Error::from("Email must be a valid email address"));
    }
     if !credentials.is_valid_email() {
        return Err(Error::from("Email must be a valid email address"));
    }

    if let Some(issue) = credentials.check_password_for_issues() {
        return Err(Error::from(issue));
    }

    let (user_exists,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM users WHERE email = ?)")
        .bind(credentials.email)
        .fetch_one(&mut *database)
        .await?;

    if user_exists {
        return Err(Error::from("The requested email is already in use"));
    }

    let new_user_id = Uuid::new_v4();
    info!(
        "Registering new user {} to uuid {}",
        credentials.email, new_user_id
    );
    
    sqlx::query("INSERT INTO users (uid, email, password_hash) VALUES (?, ?, ?)")
        .bind(new_user_id)
        .bind(credentials.email)
        .bind(&credentials.password_hash()[..])
        .execute(&mut *database)
        .await?;
     session.set_user_id(new_user_id);
     Ok(Redirect::to(uri!(index)))
}

// Register
/*
#[post("/signup", data = "<credentials>")]
pub async fn sign_up(
    session: Session<'_>,
    mut database: Connection<Sqlite>,
    credentials: Json<UserCredentials<'_>>,
) -> crate::Result<Redirect> {
    if !credentials.is_valid_email() {
        return Err(Error::from("Email must be a valid email address"));
    }

    if let Some(issue) = credentials.check_password_for_issues() {
        return Err(Error::from(issue));
    }

    let (user_exists,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM users WHERE email = ?)")
        .bind(credentials.email)
        .fetch_one(&mut *database)
        .await?;

    if user_exists {
        return Err(Error::from("The requested email is already in use"));
    }

    let new_user_id = Uuid::new_v4();
    info!(
        "Registering new user {} to uuid {}",
        credentials.email, new_user_id
    );

    sqlx::query("INSERT INTO users (uid, email, password_hash) VALUES (?, ?, ?)")
        .bind(new_user_id)
        .bind(credentials.email)
        .bind(&credentials.password_hash()[..])
        .execute(&mut *database)
        .await?;

    session.set_user_id(new_user_id);
    Ok(Redirect::to(uri!(user_homepage)))
}
*/

#[get("/logout")]
pub async fn logout(session: Session<'_>) -> crate::Result<Redirect> {
    // Change this so it redirect to the template
    session.remove_user_id();
    Ok(Redirect::to(uri!(login_page)))
}

