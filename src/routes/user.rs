use crate::database::Connection;
use crate::forms::UserCredentials;
use crate::error::Error;
use crate::session::{Session};
use log::info;
use rocket::response::Redirect;
use rocket::{get, post};
use sqlx::types::Uuid;
use sqlx::Sqlite;
use rocket::form::Form;


#[get("/")]
pub async fn index(session: Session<'_>,)->crate::Result<Redirect> {
    // Redirect to index or login depending on the session
    if session.user_id().is_none(){
        Ok(Redirect::to("/login"))
    }
    else{
        Ok(Redirect::to("/index"))
    }
}

#[post("/login", data = "<credentials>")]
pub async fn login(session: Session<'_>,mut database: Connection<Sqlite>,
                   credentials: Form<UserCredentials<'_>>) -> crate::Result<Redirect> {
    let user_id = sqlx::query_as("SELECT uid FROM users WHERE email = ? AND password_hash = ?")
        .bind(credentials.email)
        .bind(&credentials.password_hash()[..])
        .fetch_optional(&mut *database)
        .await?;

    match user_id {
        Some((id,)) => {
            session.set_user_id(id);
            Ok(Redirect::to("/index"))
        }
        // TO-DO:
        // 1. Add a flash error message
        //  2. Add logging to this too
        None => {
            Ok(Redirect::to("/login"))
        }
    }
}

#[post("/register", data = "<credentials>")]
pub async fn register(
    session: Session<'_>,
    mut database: Connection<Sqlite>,
    credentials: Form<UserCredentials<'_>>
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
    Ok(Redirect::to("/login"))
}

#[get("/logout")]
pub async fn logout(session: Session<'_>) -> crate::Result<Redirect> {
    session.remove_user_id();
    Ok(Redirect::to("/login"))
}

