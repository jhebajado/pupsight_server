use actix_multipart::{Field, Multipart};
use actix_web::{get, http::Error, post, web, HttpResponse};
use futures::TryStreamExt;

use crate::database::{SampleInsert, UserSession};
use crate::messages::samples::{SampleImage, SamplePendingList};

#[post("/upload")]
async fn post_upload(
    (database, info, mut payload): (
        web::Data<crate::database::Database>,
        crate::database::UserSession,
        Multipart,
    ),
) -> HttpResponse {
    let mut samples = Vec::new();
    while let Ok(Some(mut field)) = payload.try_next().await {
        let file_data = get_field_filedata(&mut field).await.unwrap();
        let raw = image::load_from_memory(&file_data).unwrap();
        let mut buffer = std::io::Cursor::new(Vec::<u8>::new());

        if raw.write_to(&mut buffer, image::ImageFormat::WebP).is_err() {
            return HttpResponse::UnsupportedMediaType().finish();
        }

        samples.push(SampleInsert {
            label: field.name().to_string(),
            bytes: file_data,
            owner_id: info.user_id,
        });
    }

    database.upload_samples(samples).await.into()
}

#[inline]
async fn get_field_filedata(field: &mut Field) -> Result<Vec<u8>, Error> {
    let mut buffer = Vec::<u8>::new();

    while let Some(chunk) = futures::StreamExt::next(field).await {
        let data = chunk.unwrap();
        buffer.extend_from_slice(&data);
    }

    Ok(buffer)
}

#[get("/image")]
async fn get_image(
    (database, desc): (
        web::Data<crate::database::Database>,
        web::Query<SampleImage>,
    ),
) -> HttpResponse {
    database.get_sample_image(desc.sample_id).await.into()
}

#[get("/pendings")]
async fn get_upload(
    (database, user, desc): (
        web::Data<crate::database::Database>,
        UserSession,
        web::Query<SamplePendingList>,
    ),
) -> HttpResponse {
    database
        .get_sample_list(user.user_id, desc.into_inner())
        .await
        .into()
}

#[post("/infer")]
async fn post_infer(
    (database, detector, user, desc): (
        web::Data<crate::Database>,
        web::Data<crate::Detector>,
        UserSession,
        web::Query<SampleImage>,
    ),
) -> HttpResponse {
    database
        .infer_sample_image(desc.sample_id, detector.as_ref())
        .await
        .into()
}

pub(crate) fn scope() -> actix_web::Scope {
    web::scope("/samples")
        .service(post_upload)
        .service(get_image)
        .service(get_upload)
}
