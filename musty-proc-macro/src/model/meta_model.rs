use darling::{FromDeriveInput, FromField, FromMeta};

use proc_macro2::Span;
use proc_macro_error::abort;
use quote::quote;
use syn::{Ident, Path, Type, TypePath, Visibility};

/// Attributes for fields on a model struct:
/// #[musty(id)]
#[derive(FromMeta)]
struct MustyAttrs {
    #[darling(default)]
    pub(crate) id: bool,
}

/// MongoDB-specific attributes for a model struct:
/// #[model(mongo(collection = "users"))]
#[derive(Default, FromMeta)]
#[darling(default)]
pub struct MongoAttrs {
    pub(crate) collection: Option<String>,
}

/// Attributes for a model struct:
/// #[model(mongo(...))]
#[derive(FromMeta)]
pub(crate) struct MetaModelAttr {
    mongo: Option<MongoAttrs>,
}

/// A field on a model struct
#[derive(FromField)]
pub struct MetaModelField {
    ident: Option<Ident>,
    ty: syn::Type,
    musty: Option<MustyAttrs>,
}

/// The root derive type for a model struct
#[derive(FromDeriveInput)]
#[darling(attributes(model), forward_attrs(allow, doc, cfg))]
pub(crate) struct MetaModelDerive {
    pub(crate) ident: Ident,
    pub(crate) vis: Visibility,
    pub(crate) data: darling::ast::Data<darling::util::Ignored, MetaModelField>,
}

impl MetaModelDerive {
    /// Get the type of the `id` field (or field with attribute #[musty(id)]) on the model struct
    pub(crate) fn get_model_id_type(&self) -> Path {
        let ident = &self.ident;
        let data = &self.data;

        let fields = match data {
            darling::ast::Data::Struct(fields) => fields,
            _ => abort!(ident.span(), "Model must be a struct"),
        };

        let id_field = fields.iter().find(|field| {
            field.musty.as_ref().map(|musty| musty.id).unwrap_or(false)
                || field.ident == Some(Ident::new("id", Span::call_site()))
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

    /// Re-creates the struct for the Model that had the attribute #[model(...)] macro on it
    /// This edits the id type to be `musty::prelude::Id<Self, #id_type>` and adds necessary serde attributes,
    /// and required derives (Debug, serde::Serialize, serde::Deserialize)
    fn create_model_struct(
        &self,
        id_type: &Path,
        args: &MetaModelAttr,
    ) -> proc_macro2::TokenStream {
        let ident = &self.ident;
        let data = &self.data;
        let vis = &self.vis;
        let mut id_attr = quote! { #[serde(skip)] };

        if args.mongo.is_some() {
            id_attr = quote! { #[serde(rename = "_id", skip_serializing_if = "musty::prelude::Id::is_none")] };
        }

        let fields = match data {
            darling::ast::Data::Struct(fields) => fields,
            _ => abort!(ident.span(), "Model must be a struct"),
        };

        let fields = fields
            .iter()
            .filter(|field| {
                !field.musty.as_ref().map(|musty| musty.id).unwrap_or(false)
                    && field.ident != Some(Ident::new("id", Span::call_site()))
            })
            .map(|field| {
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
        }
        .into()
    }

    /// Expands the model struct and the `Model` trait implementation for the model struct
    /// If the `mongo` attribute is present, this also expands the `MongoModel` trait implementation
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
            let mongo_model = super::mongo_model::expand_mongo_model(&self, &mongo_attrs);

            model = quote! {
                #model

                #mongo_model
            };
        }

        quote! {
            #model_struct
            #model
        }
        .into()
    }
}
