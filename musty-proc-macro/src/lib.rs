use darling::FromDeriveInput;
use model::meta_model::MetaModelDerive;
use proc_macro::{self, TokenStream};
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

mod model;
mod util;

/// Reconstructs model struct and derives `Model` (and database-specific model traits).
/// 
/// Usage:
/// ```
/// use musty::prelude::*;
/// #[model(mongo(collection = "users"))]
/// struct Users {
///     #[musty(id)] // this is optional, the macro looks for a field with this attribute or named "id"
///     id: ObjectId,
///     name: String
/// }
/// ```
/// 
/// this derives `serde::Serialize`, `serde::Deserialize`, `Debug`, and adds the necessary serde attributes to the struct and id field.
/// the id field is also changed to be of type `musty::prelude::Id<Self, I>`, where `I` is the type of your `id` field (in this case: `ObjectId`)
#[proc_macro_derive(Model, attributes(musty))]
#[proc_macro_error]
pub fn model(stream: TokenStream) -> TokenStream {
    let meta_model =
        match MetaModelDerive::from_derive_input(&parse_macro_input!(stream as DeriveInput)) {
            Ok(m) => m,
            Err(e) => return TokenStream::from(e.write_errors()),
        };

    meta_model.expand()
}

#[proc_macro_derive(Filter, attributes(musty))]
#[proc_macro_error]
pub fn filter(stream: TokenStream) -> TokenStream {
    let meta_model =
        match MetaModelDerive::from_derive_input(&parse_macro_input!(stream as DeriveInput)) {
            Ok(m) => m,
            Err(e) => return TokenStream::from(e.write_errors()),
        };

    meta_model.expand_filter().into()
}