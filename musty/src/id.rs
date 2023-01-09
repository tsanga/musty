use std::marker::PhantomData;

pub struct Id<M, I: ToString = String> {
    inner: Option<I>,
    _marker: PhantomData<M>,
}

#[cfg(feature = "bson")]
mod bson {
    use bson::oid::ObjectId;

    use super::*;
    impl<M, I: ToString> From<ObjectId> for Id<M, I> {
        fn from(id: ObjectId) -> Self {
            Self {
                inner: Some(id.to_hex()),
                _marker: PhantomData,
            }
        }
    }

    impl<M, I: ToString> TryFrom<Id<M, I>> for ObjectId {
        type Error = bson::oid::Error;
        fn from(id: Id<M, I>) -> Result<Self, Self::Error> {
            if let Some(id_str) = &id.inner {
                ObjectId::from_str(id_str)?
            } else {
                Err(Self::Error::InvalidHexStringLength{ length: 0, hex: "".to_string() })
            }
        }
    }
}