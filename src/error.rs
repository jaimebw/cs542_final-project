use log::error;
use rocket::http::Status;
use rocket::response::Responder;
use rocket::Request;
use std::borrow::Cow;

pub type MixedResult<T> = Result<T, Error>;

/// An error type why can be created from an sqlx error and respond with error text on a bad
/// request. When created from an sqlx error, the error will be logged on the server and the
/// requester will be sent an internal server error.
pub enum Error {
    BadRequest(Cow<'static, str>),
    SqlError(sqlx::Error),
}

impl<'r, 'o: 'r> Responder<'r, 'o> for Error {
    #[track_caller]
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'o> {
        match self {
            Error::BadRequest(err) => (Status::BadRequest, err).respond_to(request),
            Error::SqlError(err) => {
                error!(
                    "{} {}: Encountered SQLx error: {}",
                    request.method(),
                    request.uri().path(),
                    err
                );
                Err(Status::InternalServerError)
            }
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(error: sqlx::Error) -> Self {
        Error::SqlError(error)
    }
}

impl From<&'static str> for Error {
    fn from(error: &'static str) -> Self {
        Error::BadRequest(Cow::from(error))
    }
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error::BadRequest(Cow::from(error))
    }
}
