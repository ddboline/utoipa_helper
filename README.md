# utoipa_helper

This crate provides a derive macro `UtoipaResponse` which will implement both an `axum::response::IntoResponse` implementation and a `utoipa::IntoResponses` implementation for a new-type struct with a single set of configurations.