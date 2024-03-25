use actix_web::dev::Payload;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::HttpRequest;

use base64::prelude::*;
use diesel::Insertable;

use crate::schema::users;

use crate::password_hasher::PasswordHash;

#[derive(Clone, Debug, PartialEq, Eq, Insertable)]
#[diesel(table_name = users)]
pub(crate) struct UserInsert {
    pub(crate) login_name: String,
    pub(crate) first_name: String,
    pub(crate) last_name: String,
    pub(crate) argon2: PasswordHash,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct UserSession {
    pub(crate) user_id: uuid::Uuid,
    pub(crate) login_name: String,
    pub(crate) first_name: String,
    pub(crate) last_name: String,
}

#[derive(Debug)]
pub(crate) struct AuthorizationError;

impl actix_web::FromRequest for UserSession {
    type Error = AuthorizationError;

    type Future =
        std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self, AuthorizationError>>>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let req = req.clone();

        Box::pin(async move {
            let session_id = req
                .cookie("session")
                .ok_or(AuthorizationError)?
                .value()
                .to_owned()
                .parse::<uuid::Uuid>()
                .or(Err(AuthorizationError))?;

            let access_token = BASE64_STANDARD
                .decode(
                    req.cookie("access_token")
                        .ok_or(AuthorizationError)?
                        .value(),
                )
                .or(Err(AuthorizationError))?;

            let database = req.app_data::<Data<super::Database>>().unwrap();

            database.get_user_session(session_id, access_token).await
        })
    }
}

impl actix_web::ResponseError for AuthorizationError {
    fn status_code(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let mut res = actix_web::HttpResponse::new(self.status_code());

        res.headers_mut().insert(
            actix_web::http::header::CONTENT_TYPE,
            actix_web::http::header::HeaderValue::from_static("application/json"),
        );

        res
    }
}

impl std::fmt::Display for AuthorizationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Authorization error")
    }
}
