use actix_web::HttpResponse;

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
