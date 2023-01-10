use std::{marker::PhantomData, pin::Pin, task::Poll};

use async_trait::async_trait;
use bson::Document;
use futures::Stream;
use mongodb::{
    options::{
        CollectionOptions, FindOneAndReplaceOptions, FindOneOptions, FindOptions, ReadConcern,
        ReturnDocument, SelectionCriteria, WriteConcern,
    },
    Collection, Database,
};

use crate::{db::Db, model::Model};
use crate::{
    error::MustyError,
    id::IdType,
    prelude::{Id, Identifiable, Result},
};

#[async_trait]
pub trait MongoModel<I: IdType + Into<bson::Bson>>
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

    fn document_from_model(&self) -> Result<Document> {
        match bson::to_bson(&self)? {
            bson::Bson::Document(doc) => Ok(doc),
            _ => Err(MustyError::Other(anyhow::anyhow!(
                "Could not convert model to document"
            ))),
        }
    }

    fn model_from_document(document: Document) -> Result<Self> {
        Ok(bson::from_bson::<Self>(bson::Bson::Document(document))?)
    }

    async fn get_by_id<II: Into<Id<Self, I>> + Send>(
        db: &Db<Database>,
        id: II,
    ) -> Result<Option<Self>> {
        let filter = bson::doc! { "_id": id.into() };
        Ok(Self::collection(db).find_one(filter, None).await?)
    }

    async fn find<F, O>(db: &Db<Database>, filter: F, options: O) -> Result<MongoCursor<I, Self>>
    where
        F: Into<Option<Document>> + Send,
        O: Into<Option<FindOptions>> + Send,
    {
        Ok(Self::collection(db)
            .find(filter, options)
            .await
            .map(MongoCursor::new)?)
    }

    async fn find_one<F, O>(db: &Db<Database>, filter: F, options: O) -> Result<Option<Self>>
    where
        F: Into<Option<Document>> + Send,
        O: Into<Option<FindOneOptions>> + Send,
    {
        Ok(Self::collection(db).find_one(filter, options).await?)
    }

    async fn save(&mut self, db: &Db<Database>) -> Result<()> {
        let collection = Self::collection(db);

        let mut write_concern = Self::write_concern().unwrap_or_default();
        write_concern.journal = Some(true);

        let find_options = FindOneAndReplaceOptions::builder()
            .upsert(Some(true))
            .write_concern(Some(write_concern))
            .return_document(Some(ReturnDocument::After))
            .build();

        let filter = match &self.id().inner {
            Some(_) => bson::doc! { "_id": &self.id() },
            None => bson::doc! {},
        };

        println!("filter: {:?}", filter);

        let model = collection
            .find_one_and_replace(filter, &(*self), Some(find_options))
            .await?
            .ok_or(MustyError::MongoServerFailedToReturnUpdatedDoc)?;

        let updated_oid = model.id().clone();
        self.set_id(updated_oid);
        Ok(())
    }

    /*async fn delete(&self, db: &Db<Database>) -> Result<DeleteResult> {
        let id = self.id();
        Ok(Self::collection(db).delete_one(bson::doc! { "_id": "" }, None).await?)
    }*/
}

#[async_trait]
impl<I, M> Identifiable<I, M, Database> for Id<M, I>
where
    I: IdType + Send + Sync + Into<bson::Bson>,
    M: MongoModel<I> + Send + Sync,
{
    async fn get(self, _db: &crate::db::Db<mongodb::Database>) -> Result<Option<M>> {
        M::get_by_id(_db, self).await.map_err(|err| err.into())
    }
}

pub struct MongoCursor<I, M>
where
    I: IdType,
    M: Model<I>,
{
    cursor: mongodb::Cursor<M>,
    _marker: PhantomData<I>,
}

impl<I, M> Unpin for MongoCursor<I, M>
where
    I: IdType,
    M: Model<I>,
{
}

impl<I, M> MongoCursor<I, M>
where
    I: IdType,
    M: Model<I>,
{
    pub fn new(cursor: mongodb::Cursor<M>) -> Self {
        Self {
            cursor,
            _marker: PhantomData,
        }
    }
}

impl<I, M> Stream for MongoCursor<I, M>
where
    I: IdType,
    M: Model<I>,
{
    type Item = Result<M>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let model = match Pin::new(&mut self.cursor).poll_next(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(None) => return Poll::Ready(None),
            Poll::Ready(Some(Err(err))) => return Poll::Ready(Some(Err(MustyError::from(err)))),
            Poll::Ready(Some(Ok(model))) => model,
        };

        Poll::Ready(Some(Ok(model)))
    }
}
