use crate::prelude::Model;
use async_trait::async_trait;
use std::ops::Deref;

pub struct Db<T> {
    pub(crate) inner: T,
}

#[cfg(feature = "mongodb")]
impl<T> Db<T>
where
    T: Into<mongodb::Database>,
{
    pub fn mongo(db: T) -> Db<mongodb::Database> {
        Db { inner: db.into() }
    }
}

#[async_trait]
pub trait Identifable<I, M, D>
where
    I: ToString + Send + Sync,
    M: Model<I> + Send + Sync,
    D: Send + Sync,
{
    async fn get_model(self, db: &Db<D>) -> std::result::Result<M, crate::error::MustyError>;
}
