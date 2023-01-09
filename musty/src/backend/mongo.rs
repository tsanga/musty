use mongodb::{options::{ReadConcern, WriteConcern, SelectionCriteria, CollectionOptions}, Database, Collection};

use crate::{db::Db, model::Model};


pub trait MongoModel<I: ToString>
where Self: Model<I>
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
                .build()
        )
    }

    async fn find_one<F, O>(db: &Db<Database>, filter: F, options: O) -> Result<Option<Self>>
    where 
        F: Into<Option<Document>,
        O: Into<Option<FindOneOptions>,
    {
        Ok(Self::collection(db).find_one(filter, options).await?)
    }
}