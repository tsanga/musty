use std::panic;

use async_trait::async_trait;
use mongodb::{
    options::{CollectionOptions, ReadConcern, SelectionCriteria, WriteConcern},
    Collection, Database,
};



use crate::prelude::Id;
use crate::{db::Db, model::Model, prelude::Identifable};

#[async_trait]
pub trait MongoModel<I: ToString>
where
    Self: Model<I>,
{
    const COLLECTION_NAME: &'static str;

    fn read_concern() -> Option<ReadConcern> {
        None
    }

    fn write_concern() -> Option<WriteConcern> {
        None
    }

    fn selection_criteria() -> Option<SelectionCriteria> {
        None
    }

    fn collection(db: &Db<Database>) -> Collection<Self> {
        db.inner.collection_with_options(
            Self::COLLECTION_NAME,
            CollectionOptions::builder()
                .selection_criteria(Self::selection_criteria())
                .read_concern(Self::read_concern())
                .write_concern(Self::write_concern())
                .build(),
        )
    }
}

#[async_trait]
impl<I, M> Identifable<I, M, Database> for Id<M, I>
where
    I: ToString + Send + Sync,
    M: Model<I> + Send + Sync,
{
    async fn get_model(
        self,
        _db: &crate::db::Db<mongodb::Database>,
    ) -> std::result::Result<M, crate::error::MustyError> {
        panic!("not implemented")
    }
}
