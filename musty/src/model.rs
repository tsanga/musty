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
