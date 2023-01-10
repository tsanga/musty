use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::marker::PhantomData;

pub type DefaultType = String;
pub trait Generated: IdType {}

pub trait IdType: ToString + Serialize + DeserializeOwned + Clone + Send + Sync {}
impl<T: ToString + Serialize + DeserializeOwned + Clone + Send + Sync> IdType for T {}

#[derive(Debug)]
pub struct Id<M, I: IdType = DefaultType> {
    pub(crate) inner: Option<I>,
    _marker: PhantomData<M>,
}

impl<M, I: IdType> Id<M, I> {
    pub fn is_none(&self) -> bool {
        self.inner.is_none()
    }
}

impl<M, I: IdType> Clone for Id<M, I> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _marker: PhantomData,
        }
    }
}

impl<M, I: IdType> Default for Id<M, I> {
    fn default() -> Self {
        Self {
            inner: None,
            _marker: PhantomData,
        }
    }
}

impl<M, I: IdType> From<I> for Id<M, I> {
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

impl<M, I: IdType> Serialize for Id<M, I> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de, M, I: IdType> Deserialize<'de> for Id<M, I> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id = Deserialize::deserialize(deserializer)?;
        Ok(Self {
            inner: id,
            _marker: PhantomData,
        })
    }
}

#[cfg(feature = "bson")]
mod bson {
    use crate::error::MustyError;

    use super::*;
    use ::bson::{oid::ObjectId, Bson};

    impl Generated for ObjectId {}

    impl<M, I: IdType> TryFrom<Id<M, I>> for ObjectId {
        type Error = MustyError;

        fn try_from(id: Id<M, I>) -> Result<Self, Self::Error> {
            match id.inner {
                Some(id) => Ok(ObjectId::parse_str(&id.to_string())?),
                None => Err(crate::error::MustyError::Other(anyhow::anyhow!(
                    "Id is None"
                ))),
            }
        }
    }

    impl<M, I: IdType> TryFrom<&Id<M, I>> for ObjectId {
        type Error = MustyError;

        fn try_from(id: &Id<M, I>) -> Result<Self, Self::Error> {
            match &id.inner {
                Some(id) => Ok(ObjectId::parse_str(&id.to_string())?),
                None => Err(crate::error::MustyError::Other(anyhow::anyhow!(
                    "Id is None"
                ))),
            }
        }
    }

    impl<M, I: IdType + Into<Bson>> From<Id<M, I>> for Bson {
        fn from(id: Id<M, I>) -> Self {
            match id.inner {
                Some(id) => id.into(),
                None => Bson::Null,
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn serialize() {}

    #[test]
    fn deserialize() {}
}
