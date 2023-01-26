use futures::Stream;

use crate::prelude::Model;

/// A simple wrapper for the cursor for musty models.
/// Used when finding multiple models
/// For MongoDB, this is a `mongodb::Cursor`
pub trait MustyCursor<M>
where
    Self: Unpin + Stream + Sized,
    M: Model,
{
}
