#[cfg(feature = "mongodb")]
mod mongo;

/// Exposes basic database-agnostic model operations.
#[async_trait]
pub trait Backend: Send + Sync + Sized {
    type Filter: Send + Sync;

    async fn get_model_by_id<C, I>(&self, id: &Id<C, I>) -> Result<Option<C>>
    where
        I: IdGuard,
        C: Context<I, Self> + Model + 'static;
    async fn save_model<C, I>(&self, model: &mut C) -> Result<bool>
    where
        I: IdGuard,
        C: Context<I, Self> + Model + 'static;
    async fn delete_model<C, I>(&self, model: &mut C) -> Result<bool>
    where
        I: IdGuard,
        C: Context<I, Self> + Model + 'static;

    async fn find_one<C, I, F>(&self, filter: F) -> Result<Option<C>>
    where
        I: IdGuard,
        C: Context<I, Self> + Model + 'static,
        F: Into<Self::Filter> + Send + Sync;
}

#[cfg(feature = "mongodb")]
pub use mongo::MongoModel;

use async_trait::async_trait;

use crate::prelude::{Context, Id, IdGuard, Model};
use crate::Result;
