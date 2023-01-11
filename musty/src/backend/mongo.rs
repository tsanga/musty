use std::{marker::PhantomData, pin::Pin, task::Poll};

use async_trait::async_trait;
use bson::Document;
use bson::Document;
use futures::Stream;
use mongodb::{
    options::{
        CollectionOptions, FindOneAndReplaceOptions, FindOneOptions, FindOptions, ReadConcern,
        ReturnDocument, SelectionCriteria, WriteConcern, UpdateModifications, FindOneAndUpdateOptions, FindOneAndDeleteOptions, DeleteOptions,
    },
    Collection, Database, results::DeleteResult,
};

use crate::{db::Db, model::Model};
use crate::{
    error::MustyError,
    id::IdType,
    prelude::{Id, Identifiable},
    Result,
};

/// The model implementation used by models which use MongoDB as a backend.
/// Automatically implemented when using `#[model(mongo)]` on a model.
#[async_trait]
pub trait MongoModel<I: IdType + Into<bson::Bson>>
where
    Self: Model<I>,
{
    /// The collection name for this model
    /// Automatically implemented
    /// Can be set using `#[model(collection_name = "name")]` on the model struct
    const COLLECTION_NAME: &'static str;

    /// The read concern for MongoDB for this collection
    fn read_concern() -> Option<ReadConcern> {
        None
    }

    /// The write concern for MongoDB for this collection
    fn write_concern() -> Option<WriteConcern> {
        None
    }

    /// The selection criteria for MongoDB for this collection
    fn selection_criteria() -> Option<SelectionCriteria> {
        None
    }

    /// The collection for this model
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

    /// Converts the model to a BSON document
    fn document_from_model(&self) -> Result<Document> {
        match bson::to_bson(&self)? {
            bson::Bson::Document(doc) => Ok(doc),
            _ => Err(MustyError::Other(anyhow::anyhow!(
                "Could not convert model to document"
            ))),
        }
    }

    /// Converts a BSON document to this model type
    fn model_from_document(document: Document) -> Result<Self> {
        Ok(bson::from_bson::<Self>(bson::Bson::Document(document))?)
    }

    /// Get an instance of this model type by `Id`
    async fn get_by_id<II: Into<Id<Self, I>> + Send>(
        db: &Db<Database>,
        id: II,
    ) -> Result<Option<Self>> {
        let filter = bson::doc! { "_id": id.into() };
        Ok(Self::collection(db).find_one(filter, None).await?)
    }

    /// Find instances of this model type that match the given filter (ex `bson::doc! { "name": "John" }`)
    /// Returns a `MongoCursor` which can be used to iterate over the results
    /// Use `futures::StreamExt` to iterate over the results using
    /// `while let Some(result) = cursor.next().await {}`
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

    /// Find one instance of this model type that matches the given filter
    /// ex `bson::doc! { "name": "John" }`
    async fn find_one<F, O>(db: &Db<Database>, filter: F, options: O) -> Result<Option<Self>>
    where
        F: Into<Option<Document>> + Send,
        O: Into<Option<FindOneOptions>> + Send,
    {
        Ok(Self::collection(db).find_one(filter, options).await?)
    }

    /// Find a single document and replace it
    async fn find_one_and_replace<F, O>(db: &Db<Database>, filter: F, replacement: &Self, options: O) -> Result<Option<Self>>
    where
        F: Into<Document> + Send,
        O: Into<Option<FindOneAndReplaceOptions>> + Send,
    {
        Ok(Self::collection(db)
            .find_one_and_replace(filter.into(), replacement, options)
            .await?)
    }

    /// Find a single document and update it
    async fn find_one_and_update<F, U, O>(db: &Db<Database>, filter: F, update: U, options: O) -> Result<Option<Self>>
    where
        F: Into<Document> + Send,
        U: Into<UpdateModifications> + Send,
        O: Into<Option<FindOneAndUpdateOptions>> + Send,
    {
        Ok(Self::collection(db)
            .find_one_and_update(filter.into(), update, options)
            .await?)
    }

    /// Find a single document and delete it
    async fn find_one_and_delete<F, O>(db: &Db<Database>, filter: F, options: O) -> Result<Option<Self>>
    where 
        F: Into<Document> + Send,
        O: Into<Option<FindOneAndDeleteOptions>> + Send,
    {
        Ok(Self::collection(db)
            .find_one_and_delete(filter.into(), options)
            .await?)
    }

    /// Deletes all documents in the collection that match the given filter
    async fn delete_many<F, O>(db: &Db<Database>, filter: F, options: O) -> Result<DeleteResult>
    where
        F: Into<Document> + Send,
        O: Into<Option<DeleteOptions>> + Send,
    {
        Ok(Self::collection(db)
            .delete_many(filter.into(), options)
            .await?)   
    }

    /// Save this model instance to the database
    /// Uses `upsert: true` with `find_one_and_replace` using the _id field of the document as a filter
    /// Updates the id field of this model instance with the new id from the database
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

        let model = collection
            .find_one_and_replace(filter, &(*self), Some(find_options))
            .await?
            .ok_or(MustyError::MongoServerFailedToReturnUpdatedDoc)?;

        let updated_oid = model.id().clone();
        self.set_id(updated_oid);
        Ok(())
    }

    /// Delete this model instance from the database
    /// Errors if this model instance does not have an id set
    async fn delete(&self, db: &Db<Database>) -> Result<mongodb::results::DeleteResult> {
        let id = self.id();
        if id.is_none() {
            return Err(MustyError::MongoModelIdRequiredForOperation)
        }
        Ok(Self::collection(db).delete_one(bson::doc! { "_id": id }, None).await?)
    }
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
