use quote::quote;

use crate::util::string::{ToPlural, ToTableCase};

use super::meta_model::{MetaModelDerive, MongoAttrs};

/// Expands the `MongoModel` for a model struct
/// This sets the collection name, based on the optional attribute value:
/// `#[model(mongo(collection = "users"))]` or the default value of the table-cased & pluralized struct name
/// (ex: `MyStruct` -> `my_structs`)
pub(crate) fn expand_mongo_model(
    meta: &MetaModelDerive,
    mongo: &MongoAttrs,
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
        use musty::prelude::async_trait;

        #[async_trait]
        #[automatically_derived]
        impl musty::prelude::MongoModel<#model_id_type> for #ident where Self: Sized {
            const COLLECTION_NAME: &'static str = #collection_name;
        }
    }
}
