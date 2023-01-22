use crate::{prelude::{Filter, ModelFilter, IdGuard, Id}, Model, reference::Ref};

#[derive(Debug, Clone)]
pub enum FilterValue {
    Id(Option<String>),
    String(String),
    Int(i32),
    BigInt(i64),
    Float(f32),
    Double(f64),
    Bool(bool),
    Object(Filter),
    Vec(Vec<FilterValue>),
}

impl From<String> for FilterValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for FilterValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<i32> for FilterValue {
    fn from(i: i32) -> Self {
        Self::Int(i)
    }
}

impl From<i64> for FilterValue {
    fn from(i: i64) -> Self {
        Self::BigInt(i)
    }
}

impl From<f32> for FilterValue {
    fn from(f: f32) -> Self {
        Self::Float(f)
    }
}

impl From<f64> for FilterValue {
    fn from(f: f64) -> Self {
        Self::Double(f)
    }
}

impl From<bool> for FilterValue {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<Filter> for FilterValue {
    fn from(f: Filter) -> Self {
        Self::Object(f)
    }
}

impl<T: Into<FilterValue>> From<Vec<T>> for FilterValue {
    fn from(v: Vec<T>) -> Self {
        Self::Vec(v.into_iter().map(|v| v.into()).collect())
    }
}

impl<T: ModelFilter> From<T> for FilterValue {
    fn from(f: T) -> Self {
        Self::Object(f.get_filter().clone())
    }
}

impl<M: Model, I: IdGuard> From<Id<M, I>> for FilterValue {
    fn from(id: Id<M, I>) -> Self {
        Self::Id(id.inner.map(|i| i.to_string()))
    }
}

impl<M: Model, I: IdGuard> From<&Id<M, I>> for FilterValue {
    fn from(id: &Id<M, I>) -> Self {
        Self::Id(id.inner.as_ref().map(|i| i.to_string()))
    }
}

impl<M: Model> From<Ref<M>> for FilterValue {
    fn from(r: Ref<M>) -> Self {
        r.take_id().into()
    }
}

impl<M: Model> From<&Ref<M>> for FilterValue {
    fn from(r: &Ref<M>) -> Self {
        r.id().into()
    }
}