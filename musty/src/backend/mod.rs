#[cfg(feature = "mongodb")]
mod mongo;

#[async_trait]
pub trait Backend: Send + Sync + Sized {
    async fn get_model_by_id<C, I>(&self, id: &Id<C, I>) -> Result<Option<C>>
    where
        I: IdType,
        C: Context<I, Self> + Model<I> + 'static;
}

#[cfg(feature = "mongodb")]
pub use mongo::MongoModel;

use async_trait::async_trait;

use crate::prelude::{Context, Id, IdType, Model};
use crate::Result;
