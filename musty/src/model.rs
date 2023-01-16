use serde::{de::DeserializeOwned, Serialize};

use crate::{db::Db, Result};

use crate::prelude::{Backend, Context, Id, IdGuard};

use async_trait::async_trait;

/// Exposes basic database operations for a model.
#[doc = include_str!("../docs/model-implementing.md")]
#[async_trait]
pub trait Model<I>
where
    Self: Sized + Send + Sync + Serialize + DeserializeOwned + Unpin,
    I: IdGuard,
{
    /// Get the ID of this model.
    fn id(&self) -> &Id<Self, I>;

    /// Set the ID of this model.
    fn set_id(&mut self, id: Id<Self, I>);

    /// Get a model by its ID from a database.
    async fn get_by_id<B, T>(db: &Db<B>, id: T) -> Result<Option<Self>>
    where
        Self: Context<I, B> + 'static,
        T: Into<Id<Self, I>> + Send + Sync,
        B: Backend,
    {
        db.inner.get_model_by_id(&id.into()).await
    }

    /// Save this model to a database.
    async fn save<B>(&mut self, db: &Db<B>) -> Result<bool>
    where
        Self: Context<I, B> + 'static,
        B: Backend,
    {
        db.inner.save_model(self).await
    }

    /// Delete this model from a database.
    async fn delete<B>(&mut self, db: &Db<B>) -> Result<bool>
    where
        Self: Context<I, B> + 'static,
        B: Backend,
    {
        db.inner.delete_model(self).await
    }

    /// Find a single model from a database by a filter.
    async fn find_one<B>(db: &Db<B>, filter: B::Filter) -> Result<Option<Self>>
    where
        Self: Context<I, B> + 'static,
        B: Backend,
    {
        db.inner.find_one(filter).await
    }
}
