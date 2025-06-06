use axum::extract::{Path, Query};
use derive_more::{From, Into};
use reqwest::StatusCode;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use utoipa::{IntoParams, OpenApi};
use uuid::Uuid;

use utoipa::{IntoResponses, PartialSchema, ToSchema};
use utoipa_helper::{
    UtoipaResponse, derive_utoipa_params, derive_utoipa_schema, derive_utoipa_test,
    html_response::HtmlResponse, json_response::JsonResponse,
};

#[derive(UtoipaResponse)]
#[response(status = OK, description = "test response")]
#[rustfmt::skip]
struct TestResponse(HtmlResponse::<String>);

#[derive(Serialize, ToSchema)]
struct TestJson {
    field: String,
    id: Uuid,
    value: Decimal,
}

#[derive(UtoipaResponse)]
#[response(status = CREATED, description = "json test")]
#[rustfmt::skip]
struct TestJsonResponse(JsonResponse::<TestJson>);

#[derive(Serialize, Deserialize, Clone, Copy)]
struct Test0 {
    a: u8,
    b: u8,
}

#[derive(Serialize, Deserialize, Clone, Copy, Into, From)]
struct Test01(Test0);

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Clone, Copy, ToSchema, IntoParams)]
#[schema(as = Test0)]
struct Test1 {
    /// fieldA
    a: u8,
    /// fieldB
    b: u8,
}

derive_utoipa_schema!(Test01, Test1);
derive_utoipa_params!(Test01, Test1);

#[derive(UtoipaResponse)]
#[response(status = OK, description = "Test Description")]
#[rustfmt::skip]
struct TestResponse0(JsonResponse::<Test01>);

#[allow(dead_code)]
#[derive(Debug, IntoResponses, ToSchema)]
enum TestError {
    #[response(status = NOT_FOUND)]
    NotFound,
    #[response(status = INTERNAL_SERVER_ERROR)]
    InternalServerError,
    #[response(status = BAD_REQUEST)]
    BadRequest,
    #[response(status = METHOD_NOT_ALLOWED)]
    MethodNotAllowed,
}

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "not found"),
            Self::InternalServerError => write!(f, "internal server error"),
            Self::BadRequest => write!(f, "bad request"),
            Self::MethodNotAllowed => write!(f, "method not allowd"),
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    info(title = "Utoipa Helper", description = "Helper Macros For Utoipa Axum"),
    components(schemas(TestJson, Test01))
)]
struct ApiDoc;

#[tokio::test]
async fn test_basic_example() {
    #[utoipa::path(get, path = "/{input}", responses(TestResponse))]
    async fn test_get(input: Path<Uuid>) -> TestResponse {
        let Path(input) = input;
        HtmlResponse::new(format!("test {input}"))
            .with_cookie("test=value")
            .into()
    }

    #[utoipa::path(get, path = "/test_json", responses(TestJsonResponse))]
    async fn test_json() -> TestJsonResponse {
        let test = TestJson {
            field: "test_field".into(),
            id: Uuid::new_v4().into(),
            value: Decimal::from_str_exact("1.234").unwrap().into(),
        };
        JsonResponse::new(test).into()
    }

    let (router, api) = utoipa_axum::router::OpenApiRouter::<()>::with_openapi(ApiDoc::openapi())
        .routes(utoipa_axum::routes!(test_get))
        .routes(utoipa_axum::routes!(test_json))
        .split_for_parts();

    let spec_json = api.to_pretty_json().unwrap();
    let spec_yaml = api.to_yaml().unwrap();

    let router = router
        .route(
            "/api/openapi.json",
            axum::routing::get(|| async move {
                (
                    StatusCode::OK,
                    [("content-type", "application/json")],
                    spec_json,
                )
            }),
        )
        .route(
            "/api/openapi.yaml",
            axum::routing::get(|| async move {
                (StatusCode::OK, [("content-type", "text/yaml")], spec_yaml)
            }),
        );

    let host = "0.0.0.0";
    let port = 54321;
    let addr: SocketAddr = format!("{host}:{port}").parse().unwrap();

    let listener = TcpListener::bind(&addr).await.unwrap();

    let task = tokio::task::spawn(async move {
        axum::serve(listener, router.into_make_service())
            .await
            .unwrap()
    });
    let id = Uuid::new_v4();

    let response = reqwest::get(format!("http://0.0.0.0:54321/{id}"))
        .await
        .unwrap();

    assert_eq!(
        response
            .headers()
            .get("set-cookie")
            .unwrap()
            .to_str()
            .unwrap(),
        "test=value"
    );

    let text = response.error_for_status().unwrap().text().await.unwrap();

    assert_eq!(text, format!("test {id}"));

    let api: Value = reqwest::get("http://0.0.0.0:54321/api/openapi.json")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let api = serde_json::to_string_pretty(&api).unwrap();

    let expected = include_str!("test_basic_example.json");

    if &api != expected {
        std::fs::write("./tests/test_basic_example.json", &api).unwrap();
    }

    assert_eq!(&api, expected);

    task.abort();
}

impl axum::response::IntoResponse for TestError {
    fn into_response(self) -> axum::response::Response {
        format!("{self}").into_response()
    }
}

#[test]
fn test_derive_utoipa_test() {
    derive_utoipa_test!(Test0, Test1);
}

#[tokio::test]
async fn test_api_spec() {
    #[utoipa::path(get, path = "/", params(Test01), responses(TestResponse0, TestError))]
    async fn test_response(query: Query<Test01>) -> Result<TestResponse0, TestError> {
        let Query(Test01(x)) = query;

        Ok(JsonResponse::new(x.into()).into())
    }

    let (_, spec) = utoipa_axum::router::OpenApiRouter::<()>::with_openapi(ApiDoc::openapi())
        .routes(utoipa_axum::routes!(test_response))
        .split_for_parts();
    let spec_json = serde_json::to_string_pretty(&spec).unwrap();

    let expected = include_str!("test_schema.json");
    if &spec_json != expected {
        std::fs::write("./tests/test_schema.json", &spec_json).unwrap();
    }
    assert_eq!(&spec_json, expected);
}
