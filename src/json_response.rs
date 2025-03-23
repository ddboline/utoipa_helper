use axum::Json;
use axum::http::header::HeaderValue;
use axum::http::header::SET_COOKIE;
use axum::response::IntoResponse;
use serde::Serialize;
use std::convert::TryFrom;
use utoipa::PartialSchema;
use utoipa::ToSchema;

pub struct JsonResponse<T>
where
    T: ToSchema + Serialize + Send,
{
    data: T,
    cookies: Option<Vec<String>>,
}

impl<T> JsonResponse<T>
where
    T: ToSchema + Serialize + Send,
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

impl<T> IntoResponse for JsonResponse<T>
where
    T: ToSchema + Serialize + Send,
{
    fn into_response(self) -> axum::response::Response {
        let mut res = Json(self.data).into_response();
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

impl<T> PartialSchema for JsonResponse<T>
where
    T: ToSchema + Serialize + Send,
{
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        T::schema()
    }
}

impl<T> ToSchema for JsonResponse<T>
where
    T: ToSchema + Serialize + Send,
{
    fn name() -> std::borrow::Cow<'static, str> {
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
