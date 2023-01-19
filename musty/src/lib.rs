#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../../README.md")]

mod backend;
mod context;
mod cursor;
mod db;
mod error;
mod id;
mod model;

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

/// Exports needed to use musty.
pub mod prelude {
    pub use crate::backend::Backend;
    pub use crate::context::Context;
    pub use crate::db::Db as Musty;
    pub use crate::error::MustyError;
    pub use crate::id::DefaultType as DefaultIdType;
    pub use crate::id::GeneratedIdGuard;
    pub use crate::id::Id;
    pub use crate::id::IdGuard;
    pub use crate::model::Model;
    #[doc(hidden)]
    pub use async_trait::async_trait;
    pub use musty_proc_macro::*;

    #[cfg(feature = "mongodb")]
    #[cfg_attr(docsrs, doc(cfg(feature = "mongodb")))]
    pub use crate::backend::MongoModel;
}
