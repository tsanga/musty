use serde::{de::DeserializeOwned, Serialize};

use crate::prelude::{Id, IdType};

pub trait Model<I>
where
    Self: Sized + Send + Sync + Serialize + DeserializeOwned + Unpin,
    I: IdType,
{
    fn id(&self) -> &Id<Self, I>;
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
