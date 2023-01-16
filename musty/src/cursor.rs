use futures::Stream;

use crate::{id::IdGuard, prelude::Model};

/// A simple wrapper for the cursor for musty models.
/// Used when finding multiple models
/// For MongoDB, this is a `mongodb::Cursor`
pub trait MustyCursor<I, M>
where
    Self: Unpin + Stream + Sized,
    I: IdGuard,
    M: Model<I>,
{
}
