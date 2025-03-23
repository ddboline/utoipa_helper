use axum::body::Body;
use axum::http::header::HeaderValue;
use axum::http::header::SET_COOKIE;
use axum::response::IntoResponse;
use std::{borrow::Cow, convert::TryFrom};
use utoipa::PartialSchema;
use utoipa::ToSchema;

pub struct HtmlResponse<T>
where
    T: ToSchema + Send,
    Body: From<T>,
{
    data: T,
    cookies: Option<Vec<String>>,
}

impl<T> HtmlResponse<T>
where
    T: ToSchema + Send,
    Body: From<T>,
{
    pub fn new(data: T) -> Self {
        Self {
            data,
            cookies: None,
        }
    }

    #[must_use]
    pub fn with_cookie(mut self, cookie: impl Into<String>) -> Self {
        if let Some(cookies) = self.cookies.as_mut() {
            cookies.push(cookie.into());
        } else {
            self.cookies = Some(vec![cookie.into()]);
        }
        self
    }
}

impl<T> IntoResponse for HtmlResponse<T>
where
    T: ToSchema + Send,
    Body: From<T>,
{
    fn into_response(self) -> axum::response::Response {
        let body: Body = self.data.into();
        let mut res = body.into_response();
        if let Some(cookies) = self.cookies {
            for cookie in cookies {
                if let Ok(value) = <HeaderValue as TryFrom<String>>::try_from(cookie) {
                    res.headers_mut().append(SET_COOKIE, value);
                }
            }
        }
        res
    }
}

impl<T> PartialSchema for HtmlResponse<T>
where
    T: ToSchema + Send,
    Body: From<T>,
{
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        T::schema()
    }
}

impl<T> ToSchema for HtmlResponse<T>
where
    T: ToSchema + Send,
    Body: From<T>,
{
    fn name() -> Cow<'static, str> {
        T::name()
    }

    fn schemas(
        schemas: &mut Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    ) {
        T::schemas(schemas);
    }
}
