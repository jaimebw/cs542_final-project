use rocket::http::{Cookie, CookieJar, Status};
use rocket::outcome::IntoOutcome;
use rocket::request::{FromRequest, Outcome};
use rocket::response::Redirect;
use rocket::time::Duration;
use rocket::{uri, Request};
use sqlx::database::HasArguments;
use sqlx::encode::IsNull;
use sqlx::types::Uuid;
use sqlx::{Database, Encode, Type};
use std::convert::Infallible;
use std::ops::Deref;

const USER_TOKEN: &str = "user_token";
const SESSION_TTL: Duration = Duration::days(3);

pub struct Session<'r> {
    jar: &'r CookieJar<'r>,
}

impl<'r> Session<'r> {
    pub fn is_logged_in(&self) -> bool {
        self.user_id().is_some()
    }

    pub fn user_id(&self) -> Option<Uuid> {
        let cookie = self.jar.get_private(USER_TOKEN)?;
        Uuid::parse_str(cookie.value()).ok()
    }

    pub fn remove_user_id(&self) {
        if let Some(cookie) = self.jar.get_private(USER_TOKEN) {
            self.jar.remove_private(cookie);
        }
    }

    pub fn set_user_id(&self, user_id: Uuid) {
        let cookie_value = user_id.to_string();

        match self.jar.get_private(USER_TOKEN) {
            Some(mut cookie) => cookie.set_value(cookie_value),
            None => {
                let cookie = Cookie::build(USER_TOKEN, cookie_value)
                    .max_age(SESSION_TTL)
                    .finish();

                self.jar.add_private(cookie)
            }
        }
    }
}

impl<'r> From<&'r CookieJar<'r>> for Session<'r> {
    fn from(jar: &'r CookieJar) -> Self {
        Session { jar }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Session<'r> {
    type Error = Infallible;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        req.guard::<&CookieJar<'r>>()
            .await
            .map(|cookies| Session { jar: cookies })
    }
}

/// A helper type that can be both be used as an sql value and input for a request. Adding this as
/// an input to a request will require the requester be logged in.
#[derive(Debug)]
#[repr(transparent)]
pub struct UserId(pub Uuid);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserId {
    type Error = Redirect;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        request
            .guard::<Session<'r>>()
            .await
            .map_failure(|_| unreachable!("Session is infallible"))
            .and_then(|session| {
                session.user_id().map(UserId).into_outcome((
                    Status::Unauthorized,
                    Redirect::to(uri!(crate::routes::render_routes::login_page)),
                ))
            })
    }
}

impl<D: Database> Type<D> for UserId
where
    Uuid: Type<D>,
{
    fn type_info() -> D::TypeInfo {
        Uuid::type_info()
    }

    fn compatible(ty: &D::TypeInfo) -> bool {
        Uuid::compatible(ty)
    }
}

impl<'q, D: Database> Encode<'q, D> for UserId
where
    Uuid: Encode<'q, D>,
{
    fn encode(self, buf: &mut <D as HasArguments<'q>>::ArgumentBuffer) -> IsNull
    where
        Self: Sized,
    {
        self.0.encode(buf)
    }

    fn encode_by_ref(&self, buf: &mut <D as HasArguments<'q>>::ArgumentBuffer) -> IsNull {
        self.0.encode_by_ref(buf)
    }

    fn produces(&self) -> Option<D::TypeInfo> {
        self.0.produces()
    }

    fn size_hint(&self) -> usize {
        self.0.size_hint()
    }
}

impl Deref for UserId {
    type Target = Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
