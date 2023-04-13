use log::error;
use rocket::http::Status;
use rocket::response::Responder;
use rocket::Request;
use std::borrow::Cow;
use rocket::response::{Flash,Redirect};
use rocket_dyn_templates::Template;

pub type MixedResult<T> = Result<T, Error>;

/// An error type why can be created from an sqlx error and respond with error text on a bad
/// request. When created from an sqlx error, the error will be logged on the server and the
/// requester will be sent an internal server error.
pub enum Error {
    BadRequest(Cow<'static, str>),
    SqlError(sqlx::Error),
    ScraperError(reqwest::Error),
    FlashError(Flash<Redirect>),
    TemplateError(Template)
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
            },
            Error::ScraperError(err) => {
                error!(
                    "{} {}: Encountered scraper error: {}",
                    request.method(),
                    request.uri().path(),
                    err
                );
                (
                    Status::InternalServerError,
                    "An error occurred while communicating with Amazon",
                )
                    .respond_to(request)
            },
            Error::FlashError(err) => err.respond_to(request),
            Error::TemplateError(err) => (Status::BadRequest,err).respond_to(request),
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

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::ScraperError(error)
    }
}

impl From<Template> for Error {
    fn from(error: Template) -> Self {
        Error::TemplateError(error)
    }
}

impl From<Flash<Redirect>> for Error {
    fn from(error: Flash<Redirect>) -> Self {
        Error::FlashError(error)
    }
}

