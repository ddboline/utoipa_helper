pub mod content_type_trait;
pub mod html_response;
pub mod json_response;
pub mod response_description_trait;
pub mod status_code_trait;

pub use utoipa_helper_macro::UtoipaResponse;

#[macro_export]
macro_rules! derive_utoipa_schema {
    ($T0:ty, $T1:ty) => {
        impl utoipa::PartialSchema for $T0 {
            fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
                <$T1>::schema()
            }
        }

        impl utoipa::ToSchema for $T0 {
            fn name() -> std::borrow::Cow<'static, str> {
                assert_eq!(std::mem::size_of::<$T0>(), std::mem::size_of::<$T1>());
                <$T1>::name()
            }
            fn schemas(
                schemas: &mut Vec<(
                    String,
                    utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
                )>,
            ) {
                <$T1>::schemas(schemas)
            }
        }
    };
}

#[macro_export]
macro_rules! derive_utoipa_params {
    ($T0:ty, $T1:ty) => {
        impl utoipa::IntoParams for $T0 {
            fn into_params(
                parameter_in_provider: impl Fn() -> Option<utoipa::openapi::path::ParameterIn>,
            ) -> Vec<utoipa::openapi::path::Parameter> {
                <$T1>::into_params(parameter_in_provider)
            }
        }
    };
}

#[macro_export]
macro_rules! derive_utoipa_test {
    ($T0:ty, $T1:ty) => {
        assert_eq!(std::mem::size_of::<$T0>(), std::mem::size_of::<$T1>());
    };
}
