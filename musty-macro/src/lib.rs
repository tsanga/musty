use darling::FromDeriveInput;
use derive::MetaModelDerive;
use proc_macro::{self, TokenStream};
use syn::{DeriveInput, parse_macro_input};  
use proc_macro_error::{
    proc_macro_error
};

mod derive;

#[proc_macro_derive(Model, attributes(model))]
#[proc_macro_error]
pub fn derive_model(stream: TokenStream) -> TokenStream {
    let meta_model = match MetaModelDerive::from_derive_input(&parse_macro_input!(stream as DeriveInput)) {
        Ok(m) => m,
        Err(e) => return TokenStream::from(e.write_errors()),
    };
    meta_model.expand()
}