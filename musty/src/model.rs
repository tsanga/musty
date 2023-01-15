use serde::{de::DeserializeOwned, Serialize};

use crate::{db::Db, Result};

use crate::prelude::{Backend, Context, Id, IdType};

use async_trait::async_trait;

/// The root model type for musty models
/// Automatically derived by `musty-proc-macro` using the `#[model]` attribute
/// Using the `#[model]` attribute will also automatically change your `id` field to conform to musty's `Id` type:
/// `Id<Self, I>` where `I` is the type of your `id` field
/// If you have the `mongodb` featue enabled, you can use the `#[model(mongo)]` attribute to automatically derive `MongoModel`
#[async_trait]
pub trait Model<I>
where
    Self: Sized + Send + Sync + Serialize + DeserializeOwned + Unpin,
    I: IdType,
{
    /// The id of your model
    /// For MongoDB implementations, this is the `_id` field
    /// When using the `mongodb` feature with the `#[model(mongo)]` attribute, this field will be automatically changed to `Id<Self, I>`
    /// and renamed to `_id`.  This field will also skip serializing if the inner value is `None`
    /// This function is automatically implemented when using the `#[model]` attribute macro
    fn id(&self) -> &Id<Self, I>;

    /// Set the id of your model
    /// This function is automatically implemented when using the `#[model]` attribute macro
    fn set_id(&mut self, id: Id<Self, I>);

    async fn get_by_id<B, T>(db: &Db<B>, id: T) -> Result<Option<Self>>
    where
        Self: Context<I, B> + 'static,
        T: Into<Id<Self, I>> + Send + Sync,
        B: Backend,
    {
        db.inner.get_model_by_id(&id.into()).await
    }

    async fn save<B>(&mut self, db: &Db<B>) -> Result<()>
    where
        Self: Context<I, B> + 'static,
        B: Backend,
    {
        db.inner.save_model(self).await?;
        Ok(())
    }
}
