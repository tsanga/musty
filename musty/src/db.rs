use std::ops::Deref;

use async_trait::async_trait;
use bson::Document;

pub struct Db<T> {
    pub(crate) inner: T
}

#[cfg(feature = "mongodb")]
impl Db<T> {
    pub fn mongo(db: mongodb::Database) -> Self {
        Self {
            inner: db
        }
    }
}
