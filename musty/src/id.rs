use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::marker::PhantomData;

use crate::Model;

/// Default ID type to use if none is specified.
pub type DefaultType = String;

/// Denotes an ID that can be generated by the database.
pub trait GeneratedIdGuard: IdGuard {}

/// Guards the underlying type in an [`Id`].
pub trait IdGuard: ToString + Serialize + DeserializeOwned + Clone + Send + Sync + PartialEq {}
impl<T: ToString + Serialize + DeserializeOwned + Clone + Send + Sync + PartialEq> IdGuard for T {}

/// Wrapper struct for a model ID, which holds the model type.
#[derive(Debug, PartialEq)]
pub struct Id<M: Model, I: IdGuard = <M as Model>::Id> {
    /// The inner value of the id type, optional.  If none, serializing will skip this field.  If the type is ObjectId, this will be generated by the MongoDB server.
    pub(crate) inner: Option<I>,
    _marker: PhantomData<M>,
}

impl<M: Model, I: IdGuard> Id<M, I> {
    pub fn is_none(&self) -> bool {
        self.inner.is_none()
    }
}

impl<M: Model, I: IdGuard> Clone for Id<M, I> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _marker: PhantomData,
        }
    }
}

impl<M: Model, I: IdGuard> Default for Id<M, I> {
    fn default() -> Self {
        Self {
            inner: None,
            _marker: PhantomData,
        }
    }
}

impl<M: Model, I: IdGuard> From<I> for Id<M, I> {
    fn from(id: I) -> Self {
        Self {
            inner: Some(id),
            _marker: PhantomData,
        }
    }
}

impl<M, I> Id<M, I>
where
    M: Model,
    I: GeneratedIdGuard,
{
    pub fn none() -> Self {
        return Self {
            inner: None,
            _marker: PhantomData,
        };
    }
}

impl<M: Model, I: IdGuard> Serialize for Id<M, I> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.inner.serialize(serializer)
    }
}

impl<'de, M: Model, I: IdGuard> Deserialize<'de> for Id<M, I> {
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
#[cfg_attr(docsrs, doc(cfg(any(feature = "bson", feature = "mongodb"))))]
mod bson {
    use crate::error::MustyError;

    use super::*;
    use ::bson::{oid::ObjectId, to_bson, Bson};

    impl GeneratedIdGuard for ObjectId {}

    impl<M: Model, I: IdGuard> TryFrom<Id<M, I>> for ObjectId {
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

    impl<M: Model, I: IdGuard> TryFrom<&Id<M, I>> for ObjectId {
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

    impl<M, I> TryFrom<Id<M, I>> for Bson
    where
        I: IdGuard,
        M: Model,
    {
        type Error = MustyError;

        fn try_from(id: Id<M, I>) -> Result<Self, Self::Error> {
            match id.inner {
                Some(id) => bson::to_bson(&id).map_err(|e| MustyError::Other(e.into())),
                None => Ok(Bson::Null),
            }
        }
    }

    impl<M, I> TryFrom<&Id<M, I>> for Bson
    where
        I: IdGuard,
        M: Model
    {
        type Error = MustyError;

        fn try_from(id: &Id<M, I>) -> Result<Self, Self::Error> {
            match &id.inner {
                Some(id) => bson::to_bson(id).map_err(|e| MustyError::Other(e.into())),
                None => Ok(Bson::Null),
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
