use darling::{FromDeriveInput, FromMeta};
use derive::{MetaModelAttr, MetaModelDerive};
use proc_macro::{self, TokenStream};
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, AttributeArgs, DeriveInput};

mod derive;
mod model;
mod util;

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
