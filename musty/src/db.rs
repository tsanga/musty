use crate::prelude::Backend;

/// The database type that musty uses.
/// For `mongodb` feature usages, this type will be `Db<mongodb::Database>`
/// This is a simple wrapper type to allow support for databases other than MongoDB in the future.
pub struct Db<T: Backend> {
    pub(crate) inner: T,
}

#[cfg(feature = "mongodb")]
impl<T: Backend> Db<T>
where
    T: Into<mongodb::Database>,
{
    pub fn new(db: T) -> Db<mongodb::Database> {
        Db { inner: db.into() }
    }
}
