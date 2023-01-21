use crate::{
    prelude::{Backend, IdGuard},
    Result,
};
use std::any::Any;

/// Allows for contextualization of a database connection.
pub trait Context<I, T>: Send + Sync
where
    T: Backend,
    I: IdGuard,
{
    type Output: Any + Clone + Send + Sync;

    fn contextualize(context: &T) -> Self::Output;

    fn contextualize_boxed_downcast<D: 'static>(context: &T) -> Result<D> {
        let boxed: Box<dyn Any + Send> = Box::new(Self::contextualize(context));
        boxed
            .downcast::<D>()
            .map(|d| *d)
            .map_err(|_| anyhow::anyhow!("Failed to downcast boxed context.",).into())
    }
}
