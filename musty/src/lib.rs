mod backend;
mod cursor;
mod db;
mod error;
mod id;
mod model;
pub mod filter;

/// Re-exports
#[cfg(feature = "mongodb")]
pub use mongodb;
#[cfg(feature = "bson")]
pub use bson;

/// Exports
pub type Result<T> = std::result::Result<T, error::MustyError>;

/// Exports needed to use musty
/// use musty::prelude::*;
pub mod prelude {
    // Db
    pub use crate::db::Db as Musty;
    pub use crate::db::Identifiable;

    // Error
    pub use crate::error::MustyError;

    // Id
    pub use crate::id::DefaultType as DefaultIdType;
    pub use crate::id::Id;
    pub(crate) use crate::id::IdType;

    // Model
    pub use crate::model::Model;
    #[cfg(feature = "mongodb")]
    pub use crate::backend::MongoModel;

    // Filter
    pub use crate::filter::*;

    // Macros
    pub use musty_proc_macro::*;
    pub use async_trait::async_trait;
}
