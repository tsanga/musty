use darling::FromDeriveInput;
use derive::MetaModelDerive;
use proc_macro::{self, TokenStream};
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

mod derive;
mod model;
mod util;

#[proc_macro_derive(Model, attributes(model))]
#[proc_macro_error]
pub fn derive_model(stream: TokenStream) -> TokenStream {
    let meta_model =
        match MetaModelDerive::from_derive_input(&parse_macro_input!(stream as DeriveInput)) {
            Ok(m) => m,
            Err(e) => return TokenStream::from(e.write_errors()),
        };
    meta_model.expand()
}
#[proc_macro_attribute]
pub fn model(_args: TokenStream, stream: TokenStream) -> TokenStream {
    // just return the stream unmodified
    stream
}
