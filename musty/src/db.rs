use crate::prelude::{IdType, Model, Result};
use async_trait::async_trait;

pub struct Db<T: Send> {
    pub(crate) inner: T,
}

#[cfg(feature = "mongodb")]
impl<T: Send> Db<T>
where
    T: Into<mongodb::Database>,
{
    pub fn mongo(db: T) -> Db<mongodb::Database> {
        Db { inner: db.into() }
    }
}

#[async_trait]
pub trait Identifiable<I, M, D>
where
    I: IdType,
    M: Model<I> + Send + Sync,
    D: Send + Sync,
{
    async fn get(self, db: &Db<D>) -> Result<Option<M>>;
}
