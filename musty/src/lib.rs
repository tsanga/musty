#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

mod backend;
mod context;
mod cursor;
mod db;
mod error;
mod id;
mod model;
pub mod filter;
mod reference;

#[cfg(feature = "bson")]
pub use bson;
/// Re-exports
#[cfg(feature = "mongodb")]
pub use mongodb;

/// Result type used by musty.
pub type Result<T> = std::result::Result<T, error::MustyError>;

pub use error::MustyError;

#[cfg(feature = "mongodb")]
#[cfg_attr(docsrs, doc(cfg(feature = "mongodb")))]
pub use backend::MongoModel;
pub use model::Model;

pub use crate::db::Db as Musty;

/// Exports needed to use musty.
pub mod prelude {
    // Db
    pub use crate::backend::Backend;
    pub use crate::context::Context;
    pub use crate::db::Db as Musty;
    pub use crate::error::MustyError;

    // Id
    pub use crate::id::DefaultType as DefaultIdType;
    pub use crate::id::GeneratedIdGuard;
    pub use crate::id::Id;
    pub use crate::id::IdGuard;

    // Model
    pub use crate::model::Model;
    #[cfg(feature = "mongodb")]
    #[cfg_attr(docsrs, doc(cfg(feature = "mongodb")))]
    pub use crate::backend::MongoModel;

    // Filter
    pub use crate::filter::*;

    // Macros
    pub use musty_proc_macro::*;
    #[doc(hidden)]
    pub use async_trait::async_trait;
}
