use crate::prelude::Backend;

/// Wrapper struct for a database connection.
#[derive(Clone)]
pub struct Db<T: Backend> {
    pub(crate) inner: T,
}

#[cfg(feature = "mongodb")]
#[cfg_attr(docsrs, doc(cfg(feature = "mongodb")))]
impl<T: Backend> Db<T>
where
    T: Into<mongodb::Database>,
{
    pub fn new(db: T) -> Db<mongodb::Database> {
        Db { inner: db.into() }
    }
}

#[cfg(feature = "mongodb")]
impl From<mongodb::Database> for Db<mongodb::Database> {
    fn from(db: mongodb::Database) -> Self {
        Db { inner: db }
    }
}
