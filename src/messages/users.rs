use serde_json::json;
use uuid::Uuid;

use actix_web::{cookie::Cookie, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct RegisterUser {
    pub(crate) login_name: String,
    pub(crate) first_name: String,
    pub(crate) last_name: String,
    pub(crate) password: String,
}

pub(crate) enum RegisterUserResult {
    LoginNameAlreadyExists,
    ServerError,
    Success { id: Uuid },
}

impl From<RegisterUserResult> for HttpResponse {
    fn from(val: RegisterUserResult) -> Self {
        match val {
            RegisterUserResult::LoginNameAlreadyExists => HttpResponse::Conflict().json(json!({
                "login_name": "Already exists"
            })),
            RegisterUserResult::ServerError => HttpResponse::InternalServerError().finish(),
            RegisterUserResult::Success { id } => HttpResponse::Ok().json(json!({
                "id": id
            })),
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct LoginUser {
    pub(crate) login_name: String,
    pub(crate) password: String,
}

pub(crate) enum LoginUserResult {
    Success {
        id: uuid::Uuid,
        access_token: String,
    },
    Invalid,
    ServerError,
}

impl From<LoginUserResult> for HttpResponse {
    fn from(val: LoginUserResult) -> Self {
        match val {
            LoginUserResult::Success { id, access_token } => HttpResponse::Ok()
                .cookie(Cookie::new("session", id.to_string()))
                .cookie(Cookie::new("access_token", access_token))
                .finish(),
            LoginUserResult::ServerError => HttpResponse::InternalServerError().finish(),
            LoginUserResult::Invalid => HttpResponse::Unauthorized().finish(),
        }
    }
}
