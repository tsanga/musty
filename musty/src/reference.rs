use serde::{Serialize, Deserialize};

use crate::{Model, id::{Id}, prelude::Backend, Result, context::Context};

#[derive(Debug, Clone)]
pub enum Ref<M: Model> {
    Id(Id<M, <M as Model>::Id>),
    Model(M),
}

impl<M: Model> Ref<M> {
    pub fn new(id: Id<M, <M as Model>::Id>) -> Self {
        Self::Id(id)
    }

    pub async fn get<B>(&self, db: &crate::Musty<B>) -> Result<Option<M>>
    where 
        M: Context<<M as Model>::Id, B> + 'static,
        B: Backend,
    {
        match self {
            Self::Id(id) => Ok(db.inner.get_model_by_id(id.into()).await?),
            Self::Model(model) => Ok(Some(model.clone())),
        }
    }

    pub async fn take<B>(self, db: &crate::Musty<B>) -> Result<Option<M>>
    where 
        M: Context<<M as Model>::Id, B> + 'static,
        B: Backend,
    {
        match self {
            Self::Id(id) => Ok(db.inner.get_model_by_id(&id.into()).await?),
            Self::Model(model) => Ok(Some(model)),
        }
    }

    pub fn id(&self) -> &Id<M, <M as Model>::Id> {
        match self {
            Self::Id(id) => id,
            Self::Model(model) => model.id(),
        }
    }

    pub fn take_id(self) -> Id<M, <M as Model>::Id> {
        match self {
            Self::Id(id) => id,
            Self::Model(model) => model.id().clone(),
        }
    }
}

impl<M: Model> From<Id<M, <M as Model>::Id>> for Ref<M> {
    fn from(id: Id<M, <M as Model>::Id>) -> Self {
        Ref::Id(id)
    }
}

impl<M: Model> From<Ref<M>> for Id<M, <M as Model>::Id> {
    fn from(id: Ref<M>) -> Self {
        match id {
            Ref::Id(id) => id,
            Ref::Model(model) => model.id().clone(),
        }
    }
}

impl<'a, M: Model> From<&'a Ref<M>> for &'a Id<M, <M as Model>::Id> {
    fn from(id: &'a Ref<M>) -> Self {
        match id {
            Ref::Id(id) => id,
            Ref::Model(model) => model.id(),
        }
    }
}

impl<M: Model> Serialize for Ref<M> {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.id().serialize(serializer)
    }
}

impl<'de, M: Model> Deserialize<'de> for Ref<M> {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Ref::Id(Id::deserialize(deserializer)?))
    }
}