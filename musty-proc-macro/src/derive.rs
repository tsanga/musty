use darling::{FromDeriveInput, FromField, FromMeta};

use crate::util::string::{ToPlural, ToTableCase};
use proc_macro2::Span;
use proc_macro_error::abort;
use quote::quote;
use syn::{Ident, Path, Type, TypePath, Visibility};

#[derive(FromMeta)]
struct MustyAttrs {
    #[darling(default)]
    pub(crate) id: bool,
}

#[derive(FromField)]
pub struct MetaModelField {
    ident: Option<Ident>,
    ty: syn::Type,
    musty: Option<MustyAttrs>,
}

#[derive(Default, FromMeta)]
#[darling(default)]
pub struct MongoAttrs {
    collection: Option<String>,
}

#[derive(FromDeriveInput)]
#[darling(attributes(model), forward_attrs(allow, doc, cfg))]
pub(crate) struct MetaModelDerive {
    ident: Ident,
    vis: Visibility,
    data: darling::ast::Data<darling::util::Ignored, MetaModelField>,
}   

#[derive(FromMeta)]
pub(crate) struct MetaModelAttr {
    mongo: Option<MongoAttrs>,
}

impl MetaModelDerive {
    fn get_model_id_type(&self) -> Path {
        let ident = &self.ident;
        let data = &self.data;

        let fields = match data {
            darling::ast::Data::Struct(fields) => fields,
            _ => abort!(ident.span(), "Model must be a struct"),
        };

        let id_field = fields
            .iter()
            .find(|field| { 
                field.musty.as_ref().map(|musty| musty.id).unwrap_or(false) ||
                field.ident == Some(Ident::new("id", Span::call_site())) 
            });

        if id_field.is_none() {
            abort!(ident.span(), "{} must have an `id` field", ident);
        }

        let path = match &id_field.unwrap().ty {
            Type::Path(TypePath { path, .. }) => path,
            _ => {
                abort!(ident.span(), "{} `id` field must be path", ident)
            }
        };

        return path.clone();
    }

    fn create_model_struct(&self, id_type: &Path, args: &MetaModelAttr) -> proc_macro2::TokenStream {
        let ident = &self.ident;
        let data = &self.data;
        let vis = &self.vis;
        let mut id_attr = quote!{ #[serde(skip)] };

        if args.mongo.is_some() {
            id_attr = quote!{ #[serde(rename = "_id", skip_serializing_if = "musty::prelude::Id::is_none")] };
        }

        let fields = match data {
            darling::ast::Data::Struct(fields) => fields,
            _ => abort!(ident.span(), "Model must be a struct"),
        };

        let fields = fields.iter().filter(|field| !field.musty.as_ref().map(|musty| musty.id).unwrap_or(false) &&
        field.ident != Some(Ident::new("id", Span::call_site())) ).map(|field| {
            let ident = field.ident.as_ref().unwrap();
            let ty = &field.ty;

            quote! {
                #ident: #ty
            }
        });

        quote! { 
            #[derive(Debug, serde::Serialize, serde::Deserialize)]
            #vis struct #ident {
                #id_attr
                id: musty::prelude::Id<Self, #id_type>,
                #(#fields),*
            }
        }.into()
    }

    pub fn expand(self, args: MetaModelAttr) -> proc_macro::TokenStream {
        let ident = &self.ident;

        let model_id_type = self.get_model_id_type();
        let model_struct = self.create_model_struct(&model_id_type, &args);

        let mut model = quote! {
            #[automatically_derived]
            impl musty::prelude::Model<#model_id_type> for #ident where Self: Sized {
                fn id(&self) -> &Id<Self, #model_id_type> {
                    &self.id
                }

                fn set_id(&mut self, id: Id<Self, #model_id_type>) {
                    self.id = id;
                }
            }
        };

        if let Some(mongo_attrs) = args.mongo {
            let collection_name = mongo_attrs.collection.unwrap_or_else(|| {
                ident
                    .to_string()
                    .to_table_case()
                    .to_ascii_lowercase()
                    .to_plural()
            });

            model = quote! {
                #model

                use musty::prelude::async_trait;

                #[async_trait]
                #[automatically_derived]
                impl musty::prelude::MongoModel<#model_id_type> for #ident where Self: Sized {
                    const COLLECTION_NAME: &'static str = #collection_name;
                }
            };
        }

        quote! {
            #model_struct
            #model
        }
        .into()
    }
}
