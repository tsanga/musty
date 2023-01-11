use thiserror::Error;

/// Error types used by musty
#[derive(Debug, Error)]
pub enum MustyError {
    #[cfg(feature = "mongodb")]
    #[error(transparent)]
    Mongo(#[from] mongodb::error::Error),

    #[cfg(feature = "mongodb")]
    #[error("MongoDB server failed to return the updated document")]
    MongoServerFailedToReturnUpdatedDoc,

    #[cfg(feature = "mongodb")]
    #[error("MongoDB server failed to return the ObjectID of the updated document")]
    MongoServerFailedToReturnObjectId,

    #[cfg(feature = "mongodb")]
    #[error("Model requires an ObjectID for this operation")]
    MongoModelIdRequiredForOperation,

    #[cfg(feature = "bson")]
    #[error(transparent)]
    ObjectId(#[from] bson::oid::Error),

    #[cfg(feature = "bson")]
    #[error(transparent)]
    BsonSerialization(#[from] bson::ser::Error),

    #[cfg(feature = "bson")]
    #[error(transparent)]
    BsonDeserialization(#[from] bson::de::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
