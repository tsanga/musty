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
    pub use crate::model::Model;
    pub use async_trait::async_trait;
    pub use musty_macro::*;

    #[cfg(feature = "mongodb")]
    pub use crate::backend::MongoModel;
}
