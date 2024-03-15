mod config;
mod database;
mod detector;
mod password_hasher;

use actix_multipart::{Field, Multipart};
use actix_web::{
    web::{self},
    App, Error, HttpResponse, HttpServer,
};

use config::ServerConfig;
use futures::{StreamExt, TryStreamExt};
use image::{imageops::FilterType, GenericImageView};

use database::Database;
use detector::Detector;
use password_hasher::PasswordHasher;

fn main() -> std::io::Result<()> {
    let config = ServerConfig::load();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(start(config))
}

async fn start(config: ServerConfig) -> std::io::Result<()> {
    let server_url = config.socket_addr();

    let database = web::Data::new(Database::new(&config.database_url).await);
    let detector = web::Data::new(Detector::new());
    let hasher = web::Data::new(PasswordHasher::new(config.salt));

    println!("SERVER_URL: {server_url}");

    HttpServer::new(move || {
        App::new()
            .app_data(database.clone())
            .app_data(detector.clone())
            .app_data(hasher.clone())
            .service(process_image)
    })
    .bind(server_url)?
    .run()
    .await
}

#[actix_web::post("/scan")]
async fn process_image(
    (detector, mut payload): (web::Data<Detector>, Multipart),
) -> Result<HttpResponse, Error> {
    println!("Proccessing image");
    if let Ok(Some(mut field)) = payload.try_next().await {
        let file_data = get_field_filedata(&mut field).await?;

        let image = {
            let raw = image::load_from_memory(&file_data).unwrap();
            let (width, height) = raw.dimensions();
            let size = width.min(height);
            let (center_x, center_y) = (width / 2, height / 2);
            let (x, y) = (center_x - size / 2, center_y - size / 2);

            raw.crop_imm(x, y, width, height)
                .resize_exact(640, 640, FilterType::CatmullRom)
        };

        let result = detector.infer(&image).await;

        return Ok(HttpResponse::Ok().json(result));
    }

    Ok(HttpResponse::NotAcceptable().finish())
}

pub async fn get_field_filedata(field: &mut Field) -> Result<Vec<u8>, Error> {
    let mut buffer = Vec::<u8>::new();

    while let Some(chunk) = field.next().await {
        let data = chunk.unwrap();
        buffer.extend_from_slice(&data);
    }

    Ok(buffer)
}
