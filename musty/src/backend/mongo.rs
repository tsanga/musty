use std::{marker::PhantomData, pin::Pin, task::Poll};

use async_trait::async_trait;
use bson::{Bson, Document};
use futures::Stream;
use mongodb::{
    options::{
        CollectionOptions, DeleteOptions, FindOneAndDeleteOptions, FindOneAndReplaceOptions,
        FindOneAndUpdateOptions, FindOneOptions, FindOptions, ReadConcern, ReturnDocument,
        SelectionCriteria, UpdateModifications, WriteConcern,
    },
    results::DeleteResult,
    Collection, Database,
};

use crate::{db::Db, model::Model, prelude::Context};
use crate::{error::MustyError, id::IdType, prelude::Id, Result};

use super::Backend;

#[async_trait]
impl Backend for Database {
    async fn get_model_by_id<C, I>(&self, id: &Id<C, I>) -> Result<Option<C>>
    where
        I: IdType,
        C: Context<I, Self> + Model<I> + 'static,
    {
        if let Ok(collection) = C::contextualize_boxed_downcast::<Collection<C>>(&self) {
            let id: Result<Bson> = id.try_into();
            return Ok(collection.find_one(bson::doc!("_id": id?), None).await?);
        }

        Ok(None)
    }

    /// Save this model instance to the database
    /// Uses `upsert: true` with `find_one_and_replace` using the _id field of the document as a filter
    /// Updates the id field of this model instance with the new id from the database
    async fn save_model<C, I>(&self, model: &mut C) -> Result<()>
    where
        I: IdType,
        C: Context<I, Self> + Model<I> + 'static,
    {
        if let Ok(collection) = C::contextualize_boxed_downcast::<Collection<C>>(&self) {
            // todo: copy write concern over from collection options, probably by using tuple above instead of just collection
            let mut write_concern = WriteConcern::default();
            write_concern.journal = Some(true);

            let find_options = FindOneAndReplaceOptions::builder()
                .upsert(Some(true))
                .write_concern(Some(write_concern))
                .return_document(Some(ReturnDocument::After))
                .build();

            let id: Result<Bson> = model.id().try_into();
            let filter = match &model.id().inner {
                Some(_) => bson::doc! { "_id": id? },
                None => bson::doc! {},
            };

            let updated_model = collection
                .find_one_and_replace(filter, &(*model), Some(find_options))
                .await?
                .ok_or(MustyError::MongoServerFailedToReturnUpdatedDoc)?;

            let updated_oid = updated_model.id().clone();
            model.set_id(updated_oid);

            Ok(())   
        } else {
            Err(MustyError::Other(anyhow::anyhow!(
                "Could not save model: no collection found"
            )))
        }
    }
}

impl<I, M> Context<I, Database> for M
where
    M: MongoModel<I> + 'static,
    I: IdType + Into<bson::Bson>,
{
    type Output = Collection<Self>;

    fn contextualize(db: &Database) -> Self::Output {
        db.collection(Self::COLLECTION_NAME)
    }
}

/// The model implementation used by models which use MongoDB as a backend.
/// Automatically implemented when using `#[model(mongo)]` on a model.
#[async_trait]
pub trait MongoModel<I: IdType>
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
    async fn find_one_and_replace<F, O>(
        db: &Db<Database>,
        filter: F,
        replacement: &Self,
        options: O,
    ) -> Result<Option<Self>>
    where
        F: Into<Document> + Send,
        O: Into<Option<FindOneAndReplaceOptions>> + Send,
    {
        Ok(Self::collection(db)
            .find_one_and_replace(filter.into(), replacement, options)
            .await?)
    }

    /// Find a single document and update it
    async fn find_one_and_update<F, U, O>(
        db: &Db<Database>,
        filter: F,
        update: U,
        options: O,
    ) -> Result<Option<Self>>
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
    async fn find_one_and_delete<F, O>(
        db: &Db<Database>,
        filter: F,
        options: O,
    ) -> Result<Option<Self>>
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

    /// Delete this model instance from the database
    /// Errors if this model instance does not have an id set
    async fn delete(&self, db: &Db<Database>) -> Result<mongodb::results::DeleteResult> {
        let id = self.id();
        if id.is_none() {
            return Err(MustyError::MongoModelIdRequiredForOperation);
        }

        let id: Result<Bson> = id.try_into();
        Ok(Self::collection(db)
            .delete_one(bson::doc! { "_id": id? }, None)
            .await?)
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
