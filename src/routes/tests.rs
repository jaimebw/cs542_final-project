use rand::Rng;
use rocket::http::{ContentType, Status};
use rocket::local::asynchronous::Client;
use rocket::uri;
use sqlx::{Pool, Sqlite};
use sqlx::pool::PoolConnection;
use crate::env::setup_dotenv;
use crate::build_rocket;

use serial_test::serial;
use uuid::Uuid;
use crate::forms::UserCredentials;
use crate::session::Session;


async fn create_client() -> Client {
    setup_dotenv();
    let rocket = build_rocket().await.unwrap();

    Client::tracked(rocket).await.unwrap()
}

async fn client_database(client: &Client) -> PoolConnection<Sqlite> {
    let pool: &Pool<Sqlite> = client.rocket().state().unwrap();
    pool.acquire().await.unwrap()
}

/// Creates a completely random string of characters between a and z.
fn rng_str(length: usize) -> String {
    let mut buffer = String::new();
    let mut rng = rand::thread_rng();

    for _ in 0..length {
        buffer.push(rng.gen_range('a'..'z'));
    }

    buffer
}

#[tokio::test]
#[serial]
pub async fn test_register() {
    let client = create_client().await;

    let response = client.post(uri!(crate::routes::user::register))
        .body("email=not_an_email&password=password123")
        .header(ContentType::Form)
        .dispatch().await
        .into_string().await;

    assert_eq!(response, Some("Email must be a valid email address".to_string()));
}

#[tokio::test]
#[serial]
pub async fn test_signup_new_user() {
    let client = create_client().await;

    let email = format!("{}@example.com", rng_str(10));
    let password = rng_str(16);

    let data = UserCredentials {
        email: &email,
        password: &password,
    };

    let response = client.post(uri!(crate::routes::user::register))
        .body(format!("email={}&password={}", data.email, data.password))
        .header(ContentType::Form)
        .dispatch().await
        .status();

    // Ensure that we receive a redirect
    assert_eq!(response, Status::SeeOther);

    // Fetch new entry from the database and ensure it matches the requested password
    let mut database = client_database(&client).await;
    let hash: Option<(Vec<u8>, )> = sqlx::query_as("SELECT password_hash FROM users WHERE email = ?")
        .bind(data.email)
        .bind(&data.password_hash()[..])
        .fetch_optional(&mut database)
        .await
        .unwrap();

    assert_eq!(&hash.unwrap().0[..], &data.password_hash()[..]);
}


#[tokio::test]
#[serial]
pub async fn test_login_logout() {
    let client = create_client().await;

    let email = format!("{}@example.com", rng_str(10));
    let password = rng_str(16);

    let data = UserCredentials {
        email: &email,
        password: &password,
    };

    let _ = client.post(uri!(crate::routes::user::register))
        .body(format!("email={}&password={}", data.email, data.password))
        .header(ContentType::Form)
        .dispatch().await;

    // Replace with completely new client to test login
    drop(client);
    let client = create_client().await;


    let response = client.post(uri!(crate::routes::user::login))
        .body(format!("email={}&password={}", data.email, data.password))
        .header(ContentType::Form)
        .dispatch().await;

    assert_eq!(response.status(), Status::SeeOther);

    let session = Session::from(response.cookies());

    // Fetch new entry from the database and ensure it matches the requested password
    let database = client_database(&client);
    let (user_id, ): (Uuid, ) = sqlx::query_as("SELECT uid FROM users WHERE email = ?")
        .bind(data.email)
        .fetch_one(&mut database.await)
        .await
        .unwrap();

    assert_eq!(session.user_id().unwrap(), user_id);

    let response = client.get(uri!(crate::routes::user::logout)).dispatch().await;

    assert_eq!(response.status(), Status::SeeOther);
    let session = Session::from(response.cookies());
    assert_eq!(session.user_id(), None);
}