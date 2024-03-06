use actix_multipart::Multipart;
use actix_web::{web, App, Error, HttpResponse, HttpServer};
use dotenvy::dotenv;

async fn process_image(_payload: Multipart) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().content_type("application/json").finish())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let server_url = std::env::var("SERVER_URL").expect("SERVER_URL must be set");

    println!("SERVER_URL: {server_url}");

    HttpServer::new(|| App::new().route("/scan", web::post().to(process_image)))
        .bind(server_url)?
        .run()
        .await
}
