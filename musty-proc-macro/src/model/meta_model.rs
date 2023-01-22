use darling::{FromDeriveInput, FromField, FromMeta};
use proc_macro2::Span;
use proc_macro_error::abort;
use quote::quote;
use syn::{Ident, Path, Type, TypePath, Visibility, Attribute};
use super::mongo_model::{ModelMongoAttrs, MustyMongoFieldAttrs};

/// Attributes for a model struct:
/// #[model(mongo(...))]
#[derive(Default, FromMeta)]
#[darling(default)]
pub(crate) struct MetaModelAttr {
    pub(crate) mongo: Option<ModelMongoAttrs>,
}

/// A field on a model struct
#[derive(FromField)]
#[darling(attributes(musty), forward_attrs(filter, cfg, doc, allow))]
pub(crate) struct MetaModelField {
    pub(crate) vis: syn::Visibility,
    pub(crate) ident: Option<Ident>,
    pub(crate) ty: syn::Type,
    #[darling(default)]
    pub(crate) id: bool,
    /// skip a field: #[musty(skip)]
    #[darling(default)]
    pub(crate) skip: bool,
    /// rename a field: #[musty(rename = "new_field_name")]
    #[darling(default)]
    pub(crate) rename: Option<String>,
    /// generate a getter function for a field:
    /// ```
    /// #[musty(get)]
    /// name: String
    /// ```
    /// generates:
    /// `pub async fn get_by_name(db, name) -> Result<Option<Self>>`
    #[darling(default)]
    pub(crate) mongo: Option<MustyMongoFieldAttrs>,
    pub(crate) attrs: Vec<Attribute>,
}

/// The root derive type for a model struct
#[derive(FromDeriveInput)]
#[darling(attributes(model), forward_attrs(allow, doc, cfg, derive))]
pub(crate) struct MetaModelDerive {
    pub(crate) ident: Ident,
    pub(crate) vis: Visibility,
    pub(crate) data: darling::ast::Data<darling::util::Ignored, MetaModelField>,
    pub(crate) attrs: Vec<Attribute>,
}

impl MetaModelDerive {
    /// Get the type of the `id` field (or field with attribute #[musty(id)]) on the model struct
    pub(crate) fn get_model_id(&self) -> (Visibility, Path) {
        let ident = &self.ident;
        let data = &self.data;

        let fields = match data {
            darling::ast::Data::Struct(fields) => fields,
            _ => abort!(ident.span(), "Model must be a struct"),
        };

        let id_field = fields.iter().find(|field| {
            field.id || field.ident == Some(Ident::new("id", Span::call_site()))
        });

        if id_field.is_none() {
            abort!(ident.span(), "{} must have an `id` field", ident);
        }

        let id_field = id_field.unwrap();

        let path = match &id_field.ty {
            Type::Path(TypePath { path, .. }) => path,
            _ => {
                abort!(ident.span(), "{} `id` field must be path", ident)
            }
        };

        return (id_field.vis.clone(), path.clone());
    }

    /// Re-creates the struct for the Model that had the attribute #[model(...)] macro on it
    /// This edits the id type to be `musty::prelude::Id<Self, #id_type>` and adds necessary serde attributes,
    /// and required derives (Debug, serde::Serialize, serde::Deserialize)
    fn create_model_struct(
        &self,
        id_vis: &Visibility,
        id_type: &Path,
        args: &MetaModelAttr,
    ) -> proc_macro2::TokenStream {
        let ident = &self.ident;
        let data = &self.data;
        let vis = &self.vis;
        let mut id_attr = quote! { #[serde(skip)] };

        if args.mongo.as_ref().is_some() {
            id_attr = quote! { #[serde(rename = "_id", skip_serializing_if = "musty::prelude::Id::is_none")] };
        }

        let fields = match data {
            darling::ast::Data::Struct(fields) => fields,
            _ => abort!(ident.span(), "Model must be a struct"),
        };

        let fields = fields
            .iter()
            .filter(|field| { !field.id && field.ident != Some(Ident::new("id", Span::call_site())) })
            .map(|field| {
                let attrs = &field.attrs;
                let vis = &field.vis;
                let ident = field.ident.as_ref().unwrap();
                let ty = &field.ty;
                let mut field_attr = quote! {};
                if field.skip {
                    field_attr = quote! { #[serde(skip)] }
                } else if let Some(rename) = field.rename.as_ref() {
                    field_attr = quote! { 
                        #[serde(rename = #rename)]
                    }
                }
                quote! {
                    #field_attr
                    #(
                        #attrs
                    )*
                    #vis #ident: #ty
                }
            }).collect::<Vec<_>>();

        let attrs = &self.attrs;

        quote! {
            #(
                #attrs
            )*
            #[derive(Clone, serde::Serialize, serde::Deserialize)]
            #vis struct #ident {
                #id_attr
                #id_vis id: musty::prelude::Id<#ident, #id_type>,
                #(#fields),*
            }
        }
    }

    /// Expands the model struct and the `Model` trait implementation for the model struct
    /// If the `mongo` attribute is present, this also expands the `MongoModel` trait implementation
    pub fn expand(self, args: MetaModelAttr) -> proc_macro::TokenStream {
        let ident = &self.ident;

        let (model_id_vis, model_id_type) = self.get_model_id();
        let model_struct = self.create_model_struct(&model_id_vis, &model_id_type, &args);

        let mut model = quote! {
            #[automatically_derived]
            impl musty::prelude::Model for #ident where Self: Sized {
                type Id = #model_id_type;

                fn id(&self) -> &Id<Self, #model_id_type> {
                    &self.id
                }

                fn set_id(&mut self, id: Id<Self, #model_id_type>) {
                    self.id = id;
                }
            }
        };

        if let Some(mongo_attrs) = args.mongo.as_ref() {
            let mongo_model = super::mongo_model::expand_mongo_model(&self, &mongo_attrs);

            let mongo_fields_impl = super::mongo_model::expand_mongo_fields_impl(&self);

            model = quote! {
                #model

                #mongo_model

                #mongo_fields_impl
            };
        }

        quote! {
            #model_struct
            #model
        }
        .into()
    }
}
