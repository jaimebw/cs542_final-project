use crate::database::Connection;
use crate::error::Error;
use crate::forms::UserCredentials;
use crate::session::Session;
use log::info;
use rocket::form::Form;
use rocket::response::{Redirect,Flash};
use rocket::{get, post};
use sqlx::types::Uuid;
use sqlx::Sqlite;

#[get("/")]
pub async fn index(session: Session<'_>) -> crate::Result<Redirect> {
    // Redirect to index or login depending on the session
    if session.user_id().is_none() {
        Ok(Redirect::to("/login"))
    } else {
        Ok(Redirect::to("/index"))
    }
}
    // TO-DO: this erro should be flashed in the html
#[post("/login", data = "<credentials>")]
pub async fn login(
    session: Session<'_>,
    mut database: Connection<Sqlite>,
    credentials: Form<UserCredentials<'_>>,
) -> crate::Result<Flash<Redirect>> {


    let user_id =
        sqlx::query_as("SELECT sid FROM Site_users WHERE email = ? AND password_hash = ?")
            .bind(credentials.email)
            .bind(&credentials.password_hash()[..])
            .fetch_optional(&mut *database)
            .await?;

    match user_id {
        Some((id,)) => {
            session.set_user_id(id);
            Ok(Flash::success(Redirect::to("/index"),"Successfully logged in"))
        }
        None => Ok(Flash::success(Redirect::to("/login"), "Incorrect password/user"))
    }
}

#[post("/register", data = "<credentials>")]
pub async fn register(
    session: Session<'_>,
    mut database: Connection<Sqlite>,
    credentials: Form<UserCredentials<'_>>,
) -> crate::Result<Flash<Redirect>> {
    // Some funny notes:
    // * When you register, you automatically are logged in( I didnt intend
    // to do that) 
    if !credentials.is_valid_email() {
        return Err(Error::from("Email must be a valid email address"));
    }
    // TO-DO: this erro should be flashed in the html
    if let Some(issue) = credentials.check_password_for_issues() {
        return Err(Error::from(issue));

    }

    let (user_exists,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM Site_users WHERE email = ?)")
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

    sqlx::query("INSERT INTO Site_users (sid, email, password_hash) VALUES (?, ?, ?)")
        .bind(new_user_id)
        .bind(credentials.email)
        .bind(&credentials.password_hash()[..])
        .execute(&mut *database)
        .await?;

    session.set_user_id(new_user_id);
    Ok(Flash::success(Redirect::to("/login"),"Succesfully registed"))
}

#[get("/logout")]
pub async fn logout(session: Session<'_>) -> crate::Result<Flash<Redirect>> {
    session.remove_user_id();
    Ok(Flash::success(Redirect::to("/login"), "Logged out succesfully!"))
}
