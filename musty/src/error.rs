use thiserror::Error;

#[derive(Debug, Error)]
pub enum MustyError {
    #[cfg(feature = "mongodb")]
    #[error(transparent)]
    Mongo(#[from] mongodb::error::Error),

    #[cfg(feature = "bson")]
    #[error(transparent)]
    Bson(#[from] bson::oid::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
