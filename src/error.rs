use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io error: {0}")]
    IO(#[from] std::io::Error),
    #[error("validation error: {0}")]
    Validation(#[from] crate::validation::PlaylistError),
}
