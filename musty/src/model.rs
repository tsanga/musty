use serde::{de::DeserializeOwned, Serialize};

use crate::prelude::{Id, IdType};

/// The root model type for musty models
/// Automatically derived by `musty-proc-macro` using the `#[model]` attribute
/// Using the `#[model]` attribute will also automatically change your `id` field to conform to musty's `Id` type:
/// `Id<Self, I>` where `I` is the type of your `id` field
/// If you have the `mongodb` featue enabled, you can use the `#[model(mongo)]` attribute to automatically derive `MongoModel`
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
}
