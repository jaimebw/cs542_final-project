use rocket::http::Status;
use rocket::outcome::Outcome::{Failure, Forward, Success};
use rocket::request::FromRequest;
use rocket::request::Outcome;
use rocket::{Request, State};
use sqlx::pool::PoolConnection;
use sqlx::Pool;
use std::ops::{Deref, DerefMut};

/// A database connection that can be used in routes to acquire a database handle
#[repr(transparent)]
pub struct Connection<D: sqlx::Database> {
    connection: PoolConnection<D>,
}

#[rocket::async_trait]
impl<'r, D: sqlx::Database> FromRequest<'r> for Connection<D> {
    type Error = Option<sqlx::Error>;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.guard::<&State<Pool<D>>>().await {
            Success(x) => match x.acquire().await {
                Ok(connection) => Success(Connection { connection }),
                Err(err) => Failure((Status::ServiceUnavailable, Some(err))),
            },
            Failure((status, ())) => Failure((status, None)),
            Forward(()) => Forward(()),
        }
    }
}

impl<D: sqlx::Database> Deref for Connection<D> {
    type Target = PoolConnection<D>;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}

impl<D: sqlx::Database> DerefMut for Connection<D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.connection
    }
}
