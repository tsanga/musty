use futures::Stream;

use crate::{id::IdType, prelude::Model};

pub trait MustyCursor<I, M>
where
    Self: Unpin + Stream + Sized,
    I: IdType,
    M: Model<I>,
{
}
