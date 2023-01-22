use serde::{de::DeserializeOwned, Serialize};

use crate::{db::Db, Result};

use crate::prelude::{Backend, Context, Id, IdGuard};

use async_trait::async_trait;

/// Exposes basic database operations for a model.
///
/// This trait is database-agnostic and only exposes basic database operations.
/// More complex database-specific operations are implemented in other traits, such as [`MongoModel`](crate::prelude::MongoModel).
///
/// Most of the time you should not need to implement this trait yourself. Instead, use the [`model`](crate::prelude::macro) macro.
/// Ideally, you should try to use only the generic operations provided by this trait, and avoid using database-specific operations unless absolutely necessary.
///
#[doc = include_str!("../docs/model-macro.md")]
#[async_trait]
pub trait Model
where
    Self: Sized + Send + Sync + Serialize + DeserializeOwned + Unpin + Clone,
{
    type Id: IdGuard;

    /// Get the ID of this model.
    fn id(&self) -> &Id<Self, Self::Id>;

    /// Set the ID of this model.
    fn set_id(&mut self, id: Id<Self, Self::Id>);

    /// Get a model by its ID from a database.
    async fn get_by_id<B, T>(db: &Db<B>, id: T) -> Result<Option<Self>>
    where
        Self: Context<Self::Id, B> + 'static,
        T: Into<Id<Self, Self::Id>> + Send + Sync,
        B: Backend,
    {
        db.inner.get_model_by_id(&id.into()).await
    }

    /// Save this model to a database.
    async fn save<B>(&mut self, db: &Db<B>) -> Result<bool>
    where
        Self: Context<Self::Id, B> + 'static,
        B: Backend,
    {
        db.inner.save_model(self).await
    }

    /// Delete this model from a database.
    async fn delete<B>(&mut self, db: &Db<B>) -> Result<bool>
    where
        Self: Context<Self::Id, B> + 'static,
        B: Backend,
    {
        db.inner.delete_model(self).await
    }

    /// Find a single model from a database by a filter.
    async fn find_one<B>(db: &Db<B>, filter: B::Filter) -> Result<Option<Self>>
    where
        Self: Context<Self::Id, B> + 'static,
        B: Backend,
    {
        db.inner.find_one(filter).await
    }
}
