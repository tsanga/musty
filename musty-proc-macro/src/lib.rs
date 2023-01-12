use darling::{FromDeriveInput, FromMeta};
use model::meta_model::{MetaModelAttr, MetaModelDerive};
use proc_macro::{self, TokenStream};
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, AttributeArgs, DeriveInput};

mod model;
mod util;

/// The primary macro for deriving/codegening the `Model` and (optionally) `MongoModel` trait implementations and required attributes
/// + id field on a Model struct
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
#[proc_macro_attribute]
#[proc_macro_error]
pub fn model(args: TokenStream, stream: TokenStream) -> TokenStream {
    let arg_model = match MetaModelAttr::from_list(&parse_macro_input!(args as AttributeArgs)) {
        Ok(m) => m,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let meta_model =
        match MetaModelDerive::from_derive_input(&parse_macro_input!(stream as DeriveInput)) {
            Ok(m) => m,
            Err(e) => return TokenStream::from(e.write_errors()),
        };

    meta_model.expand(arg_model)
}
