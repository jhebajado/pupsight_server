mod detector;

use actix_multipart::{Field, Multipart};
use actix_web::{
    web::{self, Data},
    App, Error, HttpResponse, HttpServer,
};
use dotenvy::dotenv;
use futures::{StreamExt, TryStreamExt};
use image::{imageops::FilterType, GenericImageView};

use detector::Detector;

fn main() -> std::io::Result<()> {
    dotenv().ok();

    let detector = Data::new(detector::Detector::new());

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(start(detector))
}

async fn start(session: Data<Detector>) -> std::io::Result<()> {
    let server_url = std::env::var("SERVER_URL").expect("SERVER_URL must be set");

    println!("SERVER_URL: {server_url}");

    HttpServer::new(move || App::new().app_data(session.clone()).service(process_image))
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
