use axum::http::StatusCode;

pub trait StatusCodeTrait: Send + Sync {
    fn status_code() -> StatusCode;
}

pub struct StatusCodeOk {}

impl StatusCodeTrait for StatusCodeOk {
    fn status_code() -> StatusCode {
        StatusCode::OK
    }
}

pub struct StatusCodeCreated {}

impl StatusCodeTrait for StatusCodeCreated {
    fn status_code() -> StatusCode {
        StatusCode::CREATED
    }
}

pub struct StatusCodeNoContent {}

impl StatusCodeTrait for StatusCodeNoContent {
    fn status_code() -> StatusCode {
        StatusCode::NO_CONTENT
    }
}

pub struct StatusCodeValue<const S: u16> {}

impl<const S: u16> StatusCodeTrait for StatusCodeValue<S> {
    fn status_code() -> StatusCode {
        StatusCode::from_u16(S).unwrap_or(StatusCode::OK)
    }
}
