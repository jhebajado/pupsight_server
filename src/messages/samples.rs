use actix_web::HttpResponse;
use serde::Deserialize;

pub(crate) enum SampleUploadResult {
    Success,
    Failed,
}

impl From<SampleUploadResult> for HttpResponse {
    fn from(val: SampleUploadResult) -> Self {
        match val {
            SampleUploadResult::Success => HttpResponse::Ok().finish(),
            SampleUploadResult::Failed => HttpResponse::UnprocessableEntity().finish(),
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct SampleImage {
    pub(crate) sample_id: uuid::Uuid,
}

pub(crate) enum SampleImageResult {
    Success { bytes: Vec<u8> },
    NotFound,
    ServerError,
}

impl From<SampleImageResult> for HttpResponse {
    fn from(val: SampleImageResult) -> Self {
        match val {
            SampleImageResult::Success { bytes } => {
                HttpResponse::Ok().content_type("image/webp").body(bytes)
            }
            SampleImageResult::NotFound => HttpResponse::NotFound().finish(),
            SampleImageResult::ServerError => HttpResponse::InternalServerError().finish(),
        }
    }
}
