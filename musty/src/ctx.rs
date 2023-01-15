use crate::Result;
use std::any::Any;

pub trait Context<I, T>: Send + Sync {
    type Context: Any + Clone + Send + Sync;

    fn contextualize(context: &T) -> Self::Context;

    fn contextualize_boxed_downcast<D: 'static>(context: &T) -> Result<D> {
        let boxed: Box<dyn Any + Send> = Box::new(Self::contextualize(context));
        boxed
            .downcast::<D>()
            .map(|d| *d)
            .map_err(|_| anyhow::anyhow!("Failed to downcast boxed context.",).into())
    }
}
