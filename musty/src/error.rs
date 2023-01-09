use thiserror::Error;

#[derive(Debug, Error)]
pub enum MustyError {
    #[cfg(feature = "mongodb")]
    #[error("{0}")]
    Mongo(#[from] mongodb::error::Error),
}