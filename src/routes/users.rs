use actix_web::{get, post, web, HttpResponse};

use crate::{
    database::Database,
    messages::users::{LoginUser, RegisterUser},
    password_hasher::PasswordHasher,
};

#[post("/register")]
async fn post_register(
    (database, hasher, desc): (
        web::Data<Database>,
        web::Data<PasswordHasher<'static>>,
        web::Json<RegisterUser>,
    ),
) -> HttpResponse {
    database
        .register(hasher.into_inner(), desc.into_inner())
        .await
        .into()
}

#[post("/login")]
async fn post_login(
    (database, hasher, desc): (
        web::Data<Database>,
        web::Data<PasswordHasher<'static>>,
        web::Json<LoginUser>,
    ),
) -> HttpResponse {
    database
        .login(hasher.into_inner(), desc.into_inner())
        .await
        .into()
}

#[get("/info")]
async fn get_info(info: crate::database::UserSession) -> HttpResponse {
    HttpResponse::Ok().json(info)
}

pub(crate) fn scope() -> actix_web::Scope {
    web::scope("/users")
        .service(post_register)
        .service(post_login)
        .service(get_info)
}
