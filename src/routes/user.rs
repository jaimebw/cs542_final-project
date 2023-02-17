//! Note(Jasper): I'm still working out the best way to use rocket with sqlx so don't look at any of
//! this too closely.

use crate::database::Connection;
use crate::session::{Session, UserId};
use log::error;
use regex::Regex;
use rocket::response::status::Unauthorized;
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::{get, post, uri, Responder};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use sqlx::types::Uuid;
use sqlx::Sqlite;

#[derive(Deserialize)]
pub struct UserCredentials<'a> {
    email: &'a str,
    password: &'a str,
}

impl<'a> UserCredentials<'a> {
    /// Check if the given email appears to conform to the address format for RFC5322
    fn is_valid_email(&self) -> bool {
        let email_regex = Regex::new("\
(?:[a-z0-9!#$%&'*+/=?^_`{|}~-]+(?:\\.[a-z0-9!#$%&'*+/=?^_`{|}~-]+)*|\"(?:[\x01-\x08\x0b\x0c\x0e-\x1f\
\x21\x23-\x5b\x5d-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])*\")@(?:(?:[a-z0-9](?:[a-z0-9-]*[a-z0-9])?\\.\
)+[a-z0-9](?:[a-z0-9-]*[a-z0-9])?|\\[(?:(?:(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9]))\\.){3}(?:\
(2(5[0-5]|[0-4][0-9])|1[0-9][0-9]|[1-9]?[0-9])|[a-z0-9-]*[a-z0-9]:(?:[\x01-\x08\x0b\x0c\x0e-\x1f\x21\
-\x5a\x53-\x7f]|\\[\x01-\x09\x0b\x0c\x0e-\x7f])+)\\])").expect("Input is valid regex");

        email_regex.is_match(self.email)
    }

    fn check_password_for_issues(&self) -> Option<&'static str> {
        let character_count = self.password.chars().count();
        if character_count < 8 {
            return Some("Password must be at least 8 characters");
        }

        if character_count > 512 {
            return Some("Password can not be more than 512 characters");
        }

        // What else?
        None
    }

    fn password_hash(&self) -> [u8; 32] {
        const SALT: [u8; 8] = [242, 94, 145, 122, 201, 1, 131, 203];

        let mut hasher = Sha256::new();
        hasher.update(SALT);
        hasher.update(self.password);

        hasher.finalize().into()
    }
}

#[get("/login")]
pub async fn login_page(session: Session<'_>, mut database: Connection<Sqlite>) -> Redirect {
    if let Some(user_id) = session.user_id() {
        let user_exists = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM users WHERE uid = ?)")
            .bind(user_id)
            .fetch_one(&mut *database)
            .await;

        if let Ok((true,)) = user_exists {
            // TODO: Navigate to homepage
        } else {
            // ID is invalid so remove it (probably from a previous debug version)
            session.remove_user_id();
        }
    }

    Redirect::to(uri!("/static/login.html"))
}

#[post("/login", data = "<credentials>")]
pub async fn login(
    session: Session<'_>,
    mut database: Connection<Sqlite>,
    credentials: Json<UserCredentials<'_>>,
) -> Result<Redirect, Unauthorized<&'static str>> {
    let user_id = sqlx::query_as(r#"SELECT uid FROM users WHERE email = ? AND password_hash = ?"#)
        .bind(credentials.email)
        .bind(&credentials.password_hash()[..])
        .fetch_optional(&mut *database)
        .await;

    match user_id {
        Ok(Some((id,))) => {
            session.set_user_id(id);
            Ok(Redirect::found(uri!(super::index_page)))
        }
        Ok(None) => Err(Unauthorized(Some(
            "Email/password combination does not match any registered user",
        ))),
        _ => Ok(todo!()),
    }
}

#[derive(Debug, Responder)]
pub enum SignUpResponse {
    Success(Redirect),
    #[response(status = 400)]
    BadRequest(&'static str),
    #[response(status = 500)]
    SqlError(()),
}

#[post("/signup", data = "<credentials>")]
pub async fn sign_up(
    session: Session<'_>,
    mut database: Connection<Sqlite>,
    credentials: Json<UserCredentials<'_>>,
) -> SignUpResponse {
    if !credentials.is_valid_email() {
        return SignUpResponse::BadRequest("Email must be a valid email address");
    }

    if let Some(issue) = credentials.check_password_for_issues() {
        return SignUpResponse::BadRequest(issue);
    }

    let user_exists = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM users WHERE email = ?)")
        .bind(credentials.email)
        .fetch_one(&mut *database)
        .await;

    match user_exists {
        Ok((true,)) => SignUpResponse::BadRequest("The requested email is already in use"),
        Ok((false,)) => {
            let new_user_id = Uuid::new_v4();
            session.set_user_id(new_user_id);
            // TODO: Add user

            SignUpResponse::Success(Redirect::to(uri!(user_homepage)))
        }
        Err(err) => {
            error!(
                "Encountered error while attempting to register user: {:?}",
                err
            );
            SignUpResponse::SqlError(())
        }
    }
}

#[get("/logout")]
pub async fn logout(session: Session<'_>) -> Redirect {
    session.remove_user_id();
    Redirect::to(uri!(login_page))
}

#[get("/home")]
pub async fn user_homepage(user: UserId) -> String {
    format!("Logged in as user: {:?}", user)
}
