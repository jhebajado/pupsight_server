use actix_web::HttpResponse;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub(crate) struct SamplePendingList {
    pub(crate) page: u32,
    pub(crate) keyword: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct PendingListEntry {
    pub(crate) id: uuid::Uuid,
    pub(crate) label: String,
    pub(crate) pet_id: Option<uuid::Uuid>,
}

#[derive(Serialize)]
pub(crate) struct PendingListData {
    items: Vec<PendingListEntry>,
    has_next: bool,
}

pub(crate) enum PendingListResult {
    Success {
        items: Vec<PendingListEntry>,
        has_next: bool,
    },
    Failed,
}

impl From<PendingListResult> for HttpResponse {
    fn from(val: PendingListResult) -> Self {
        match val {
            PendingListResult::Success { items, has_next } => {
                HttpResponse::Ok().json(PendingListData { items, has_next })
            }
            PendingListResult::Failed => HttpResponse::UnprocessableEntity().finish(),
        }
    }
}

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

pub(crate) enum SampleInferResult {
    Success,
    NotFound,
    ImageLoadError,
    ServerError,
}

impl From<SampleInferResult> for HttpResponse {
    fn from(val: SampleInferResult) -> Self {
        match val {
            SampleInferResult::Success => HttpResponse::Accepted().finish(),
            SampleInferResult::NotFound => HttpResponse::NotFound().finish(),
            SampleInferResult::ImageLoadError => HttpResponse::UnprocessableEntity().finish(),
            SampleInferResult::ServerError => HttpResponse::InternalServerError().finish(),
        }
    }
}
