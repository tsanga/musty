use std::collections::HashMap;
use super::{op::{FilterCmpOp, FilterLogicOp}, value::FilterValue};

#[derive(Debug, Clone)]
pub struct Filter {
    pub ops: HashMap<FilterLogicOp, Vec<FilterCondition>>,
    pub filters: Vec<FilterCondition>,
    pub children: HashMap<String, Filter>,
}

impl Filter {
    pub fn new() -> Self {
        Self {
            ops: HashMap::new(),
            filters: Vec::new(),
            children: HashMap::new(),
        }
    }

    pub fn add_condition(&mut self, filter: FilterCondition) {
        self.filters.push(filter);
    }

    pub fn add_op_condition(&mut self, op: FilterLogicOp, condition: FilterCondition) {
        if let Some(filters) = self.ops.iter_mut().find(|f| f.0 == &op).map(|f| f.1) {
            filters.push(condition);
        } else {
            self.ops.insert(op, vec![condition]);
        }
    }

    pub fn add_child(&mut self, name: String, filter: Filter) {
        self.children.insert(name, filter);
    }
}

#[derive(Debug, Clone)]
pub struct FilterCondition {
    pub key: String,
    pub op: FilterCmpOp,
    pub value: FilterValue,
}

impl FilterCondition {
    pub fn new(key: String, op: FilterCmpOp, value: FilterValue) -> Self {
        Self {
            key,
            op,
            value,
        }
    }
}

pub trait Filterable where Self: Sized {
    type ModelFilter: ModelFilter;
    fn filter() -> Self::ModelFilter {
        <Self::ModelFilter as ModelFilter>::new()
    }
}

pub trait ModelFilter: Sized + Clone {
    fn new() -> Self;

    fn get_filter(&self) -> &Filter;

    fn get_filter_mut(&mut self) -> &mut Filter;

    fn field<T: Into<FilterValue>>(&mut self, name: impl Into<String>) -> ModelFieldFilter<Self, T> {
        ModelFieldFilter::new(name, self)
    }

    fn child<C, Func>(&mut self, name: impl Into<String>, func: Func)
    where
        C: ModelFilter,
        Func: FnOnce(&mut C) -> &mut C,
    {
        let mut child_model_filter = C::new(); // where C is child
        func(&mut child_model_filter);
        self.get_filter_mut().add_child(name.into(), child_model_filter.get_filter().clone());
    }

    fn build(&self) -> Filter {
        self.get_filter().clone()
    }

    // TODO: add op funcs (any, all, etc)
}

pub struct ModelFieldFilter<'f, F, T>
where
    Self: Sized,
    F: ModelFilter,
{
    field_name: String,
    model_filter: &'f mut F,
    _field_type_marker: std::marker::PhantomData<T>,
}

impl<'f, F, T> ModelFieldFilter<'f, F, T>
where
    Self: Sized,
    F: ModelFilter,
    T: Into<FilterValue>,
{
    pub fn new(field_name: impl Into<String>, model_filter: &'f mut F) -> Self {
        Self {
            field_name: field_name.into(),
            model_filter,
            _field_type_marker: std::marker::PhantomData,
        }
    }

    fn condition(self, value: T, op: FilterCmpOp) -> &'f mut F {
        let condition = FilterCondition::new(self.field_name.clone(), op, value.into());
        self.model_filter.get_filter_mut().add_condition(condition);
        self.model_filter
    }

    pub fn eq(self, value: T) -> &'f mut F {
        self.condition(value, FilterCmpOp::Eq)
    }

    pub fn ne(self, value: T) -> &'f mut F {
        self.condition(value, FilterCmpOp::Ne)
    }

    pub fn gt(self, value: T) -> &'f mut F {
        self.condition(value, FilterCmpOp::Gt)
    }

    pub fn lt(self, value: T) -> &'f mut F {
        self.condition(value, FilterCmpOp::Lt)
    }

    pub fn ge(self, value: T) -> &'f mut F {
        self.condition(value, FilterCmpOp::Ge)
    }

    pub fn le(self, value: T) -> &'f mut F {
        self.condition(value, FilterCmpOp::Le)
    }

    pub fn any<Func>(self, func: Func) -> &'f mut F
    where Func: FnOnce(&mut ModelFieldVecFilter<T>) -> &mut ModelFieldVecFilter<T>
    {
        let mut vec_filter = ModelFieldVecFilter::<T>::new();
        func(&mut vec_filter);
        let condition = FilterCondition::new(self.field_name.clone(), FilterCmpOp::Eq, vec_filter.items.into());
        self.model_filter.get_filter_mut().add_op_condition(FilterLogicOp::Any, condition);
        self.model_filter
    }

    pub fn all<Func>(self, func: Func) -> &'f mut F
    where Func: FnOnce(&mut ModelFieldVecFilter<T>) -> &mut ModelFieldVecFilter<T>
    {
        let mut vec_filter = ModelFieldVecFilter::<T>::new();
        func(&mut vec_filter);
        let condition = FilterCondition::new(self.field_name.clone(), FilterCmpOp::Eq, vec_filter.items.into());
        self.model_filter.get_filter_mut().add_op_condition(FilterLogicOp::All, condition);
        self.model_filter
    }
}

pub struct ModelFieldVecFilter<T>
where
    Self: Sized,
{
    items: Vec<T>,
    _field_type_marker: std::marker::PhantomData<T>,
}

impl<T> ModelFieldVecFilter<T>
where
    Self: Sized,
{
    fn new() -> Self {
        Self {
            items: Vec::new(),
            _field_type_marker: std::marker::PhantomData,
        }
    }

    pub fn entry(&mut self, item: impl Into<T>) -> &mut Self {
        self.items.push(item.into());
        self
    }
}

/*
use musty::prelude::*;

#[model()]
struct User {
    id: u32,
    name: String,
    #[musty(nested)]
    auth: Auth,
}

#[derive(Serialize, Deserialize, Filter)]
struct Auth {
    password: String
}

// macro expansion:

#[derive(Clone)]
struct UserFilter {
    filter: MustyFilter
}

impl ModelFilter for UserFilter {
    fn new() -> Self {
        Self { filter: MustyFilter::new() }
    }

    fn new_nested<T: ModelFilter>(parent: &T, name: &str) -> Self {
        parent.add_filter(MustyFilter::new_nested(name));
        Self::new()
    }

    fn add_filter(&mut self, filter: MustyFilter) {
        // add filter to self
    }
}

impl UserFilter {
    fn id(&mut self) -> ModelFieldFilter<Self, u32> {
        ModelFieldFilter::new("id", &self)
    }

    fn auth(&mut self) -> AuthFilter {
        AuthFilter::new_nested(self, "auth")
    }
}

struct AuthFilter {}
impl AuthFilter {
    fn password(&mut self) -> ModelFieldFilter<Self, String> {
        ModelFieldFilter::new("password", &self)
    }
}

impl Filterable for User {
    type ModelFilter = UserFilter;
}

impl Filterable for Auth {
    type ModelFilter = AuthFilter;
}

async fn test() -> Result<()> {
    let filter = User::filter()
        .id().eq(1.into())
        .auth(|auth| auth.password().eq("password".into()));
}

*/