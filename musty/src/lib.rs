mod backend;
mod cursor;
mod db;
mod error;
mod id;
mod model;

pub mod prelude {
    pub use crate::db::Db as Musty;
    pub use crate::db::Identifiable;
    pub use crate::error::MustyError;
    pub use crate::id::DefaultType as DefaultIdType;
    pub use crate::id::Id;
    pub(crate) use crate::id::IdType;
    pub use crate::model::Model;
    pub use async_trait::async_trait;
    pub use musty_proc_macro::*;
    pub type Result<T> = std::result::Result<T, MustyError>;

    #[cfg(feature = "mongodb")]
    pub use crate::backend::MongoModel;
}
