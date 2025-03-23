pub trait ResponseDescriptionTrait: Send + Sync {
    fn description() -> &'static str;
}

pub struct DefaultDescription {}

#[macro_export]
macro_rules! derive_response_description {
    ($T:ty, $description:expr) => {
        impl $crate::response_description_trait::ResponseDescriptionTrait for $T {
            fn description() -> &'static str {
                $description
            }
        }
    };
}

derive_response_description!(DefaultDescription, "");
