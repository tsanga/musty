use async_trait::async_trait;
use mongodb::bson::oid::ObjectId;
use serde::{Serialize, de::DeserializeOwned};

use crate::id::Id;

#[async_trait]
pub trait Model<I>
where 
    Self: Serialize + DeserializeOwned + Unpin + Send + Sync,
    I: ToString,
{
    fn id(&self) -> Id<Self, I>;
    fn set_id(&mut self, id: Id<Self, I>);
}

/* 

#[derive(Model)]
#[model(mongo(collection = "users"), default_id = "ObjectId::new()")]
struct User {
    id: Id<Self>,
}

User::get_by_id

let id: Id<ObjectId, User> = Id::new()

let user = id.get(*&db).await?;

*/