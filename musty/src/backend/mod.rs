#[cfg(feature = "mongodb")]
mod mongo;

#[cfg(feature = "mongodb")]
pub use mongo::MongoModel;
