use darling::FromMeta;
use quote::{quote, format_ident};
use syn::Ident;
use crate::util::string::{ToPlural, ToTableCase};
use super::meta_model::MetaModelDerive;
use proc_macro_error::abort;

/// MongoDB-specific attributes for a model struct:
/// #[model(mongo(collection = "users"))]
#[derive(Default, FromMeta)]
#[darling(default)]
pub(crate) struct ModelMongoAttrs {
    pub(crate) collection: Option<String>,
}

/// Expands the `MongoModel` for a model struct
/// This sets the collection name, based on the optional attribute value:
/// `#[model(mongo(collection = "users"))]` or the default value of the table-cased & pluralized struct name
/// (ex: `MyStruct` -> `my_structs`)
pub(crate) fn expand_mongo_model(
    meta: &MetaModelDerive,
    mongo: &ModelMongoAttrs,
) -> proc_macro2::TokenStream {
    let ident = &meta.ident;

    let model_id_type = meta.get_model_id_type();

    let collection_name = mongo.collection.clone().unwrap_or_else(|| {
        ident
            .to_string()
            .to_table_case()
            .to_ascii_lowercase()
            .to_plural()
    });

    quote! {
        #[musty::prelude::async_trait]
        #[automatically_derived]
        impl musty::prelude::MongoModel<#model_id_type> for #ident where Self: Sized {
            const COLLECTION_NAME: &'static str = #collection_name;
        }
    }
}

pub(crate) fn expand_mongo_fields_impl(meta: &MetaModelDerive) -> proc_macro2::TokenStream {
    let ident = &meta.ident;

    let fields = match &meta.data {
        darling::ast::Data::Struct(fields) => fields,
        _ => abort!(ident.span(), "Model must be a struct"),
    };

    let mut field_impls = Vec::new();

    for field in fields.iter() {
        if let Some(get) = field.get.as_ref() {
            let mut field_ident = field.ident.clone().unwrap();
            let field_type = &field.ty;

            if let Some(rename) = field.rename.as_ref() {
                field_ident = Ident::new(rename, field_ident.span());
            }

            let field_name = field_ident.to_string();

            let get_by_field_name = match get.name.as_ref() {
                Some(get) => Ident::new(get, field_ident.span()),
                None => format_ident!("get_by_{}", field_name),
            };

            let func = quote! {
                pub async fn #get_by_field_name(db: &musty::prelude::Musty<musty::mongodb::Database>, #field_ident: #field_type) -> musty::Result<Option<Self>> {
                    Ok(Self::find_one(db, musty::bson::doc! { #field_name: #field_ident }, None).await?)
                }
            };
            field_impls.push(func);
        }
    }

    if !field_impls.is_empty() {
        quote! {
            impl #ident {
                #(#field_impls)*
            }
        }
    } else {
        quote!{ }
    }
}