use serde_json::json;
use uuid::Uuid;

use actix_web::HttpResponse;
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
