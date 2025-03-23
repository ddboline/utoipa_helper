pub trait ContentTypeTrait: Send + Sync {
    fn content_type() -> &'static str;
    fn content_type_header() -> &'static str;
}
pub struct ContentTypeHtml {}
pub struct ContentTypeCss {}
pub struct ContentTypeJs {}

#[macro_export]
macro_rules! derive_content_type {
    ($T:ty, $content_type:expr) => {
        impl $crate::content_type_trait::ContentTypeTrait for $T {
            fn content_type() -> &'static str {
                $content_type
            }
            fn content_type_header() -> &'static str {
                concat!($content_type, "; charset=utf-8")
            }
        }
    };
}

derive_content_type!(ContentTypeHtml, "text/html");
derive_content_type!(ContentTypeCss, "text/css");
derive_content_type!(ContentTypeJs, "text/javascript");
