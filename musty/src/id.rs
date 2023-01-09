use std::marker::PhantomData;

pub type DefaultType = String;
pub trait Generated: ToString {}

#[derive(Debug)]
pub struct Id<M, I: ToString = DefaultType> {
    inner: Option<I>,
    _marker: PhantomData<M>,
}

impl<M, I: ToString> From<I> for Id<M, I> {
    fn from(id: I) -> Self {
        Self {
            inner: Some(id),
            _marker: PhantomData,
        }
    }
}

impl<M, I> Id<M, I>
where
    I: Generated,
{
    pub fn none() -> Self {
        return Self {
            inner: None,
            _marker: PhantomData,
        };
    }
}

#[cfg(feature = "bson")]
mod bson {
    use super::*;
    use ::bson::oid::ObjectId;

    impl Generated for ObjectId {}
}
