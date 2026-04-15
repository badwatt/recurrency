pub mod auth_extractor;
pub mod date_utils;
pub mod db;
pub mod error;
pub mod password_utils;

#[derive(serde::Deserialize)]
pub struct AuthQuery {
    pub token: Option<String>,
}
