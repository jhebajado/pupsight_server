use actix_web::HttpResponse;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub(crate) struct SamplePendingList {
    pub(crate) page: u32,
    pub(crate) keyword: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct SampleInferredList {
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

#[derive(Serialize)]
pub(crate) struct InferredListData {
    items: Vec<InferredListEntry>,
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
            PendingListResult::Failed => HttpResponse::ServiceUnavailable().finish(),
        }
    }
}

#[derive(Serialize)]
pub(crate) struct InferredListEntry {
    pub id: uuid::Uuid,
    pub label: String,
    pub pet_id: Option<uuid::Uuid>,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
    pub results: Vec<InferredResultListEntry>,
}

#[derive(Serialize)]
pub(crate) struct InferredResultListEntry {
    pub id: uuid::Uuid,
    pub certainty: f32,
    pub is_normal: bool,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub iris_x: Option<f32>,
    pub iris_y: Option<f32>,
    pub iris_a: Option<f32>,
    pub iris_b: Option<f32>,
    pub coverage: Option<f32>,
    pub created_at: NaiveDateTime,
    pub updated_at: Option<NaiveDateTime>,
}

pub(crate) enum InferredListResult {
    Success {
        items: Vec<InferredListEntry>,
        has_next: bool,
    },
    Failed,
}

impl From<InferredListResult> for HttpResponse {
    fn from(val: InferredListResult) -> Self {
        match val {
            InferredListResult::Success { items, has_next } => {
                HttpResponse::Ok().json(InferredListData { items, has_next })
            }
            InferredListResult::Failed => HttpResponse::ServiceUnavailable().finish(),
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
    Reject,
    ImageLoadError,
    ServerError,
}

impl From<SampleInferResult> for HttpResponse {
    fn from(val: SampleInferResult) -> Self {
        match val {
            SampleInferResult::Success => HttpResponse::Accepted().finish(),
            SampleInferResult::Reject => HttpResponse::UnprocessableEntity().finish(),
            SampleInferResult::NotFound => HttpResponse::NotFound().finish(),
            SampleInferResult::ImageLoadError => HttpResponse::InternalServerError().finish(),
            SampleInferResult::ServerError => HttpResponse::InternalServerError().finish(),
        }
    }
}
