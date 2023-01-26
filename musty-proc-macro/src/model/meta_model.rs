use super::mongo_model::{ModelMongoAttrs, MustyMongoFieldAttrs};
use darling::{FromDeriveInput, FromField, FromMeta};
use proc_macro_error::abort;
use quote::{format_ident, quote};
use syn::{spanned::Spanned, Attribute, Ident, Type, TypePath, Visibility};

#[derive(Default, FromMeta)]
#[darling(default)]
struct SerdeFieldAttr(String);

/// A field on a model struct
#[derive(FromField)]
#[darling(attributes(musty), forward_attrs(serde, cfg, doc, allow))]
pub(crate) struct MetaModelField {
    pub(crate) vis: Visibility,
    pub(crate) ident: Option<Ident>,
    pub(crate) ty: syn::Type,
    #[darling(default)]
    pub(crate) id: bool,
    #[darling(default)]
    pub(crate) skip: bool,
    #[darling(default)]
    pub(crate) rename: Option<String>,
    #[darling(default)]
    pub(crate) mongo: Option<MustyMongoFieldAttrs>,
    #[darling(default)]
    pub(crate) child: bool,
    pub(crate) attrs: Vec<Attribute>,
}

impl MetaModelField {
    fn is_id(&self) -> bool {
        self.id || self.ident.as_ref().unwrap().to_string() == "id"
    }

    fn validate_id_field(&self, meta_model: &MetaModelDerive) {
        let ty = &self.ty;
        let path = match ty {
            Type::Path(TypePath { path, .. }) => path,
            _ => {
                abort!(ty.span(), "`id` field must be a path")
            }
        };
        let Some(last_segment) = path.segments.last() else { abort!(ty.span(), "Id type is empty") };
        if last_segment.ident.to_string() != "Id"
            && last_segment.ident.to_string() != "musty::prelude::Id"
        {
            abort!(ty.span(), "`id` field must be of type `musty::prelude::Id`")
        }

        let syn::PathArguments::AngleBracketed(path_arguments) = &last_segment.arguments else { abort!(ty.span(), "Id type must have generic arguments `<Self, YourIdType>` where `YourdIdType` implements `musty::prelude::IdGuard`") };

        let syn::GenericArgument::Type(syn::Type::Path(syn::TypePath { path: id_model_path, .. })) = path_arguments.args.first().unwrap() else { abort!(path_arguments.span(), "First generic argument of `Id` type should be `Self`: `Id<Self, ..>`") };

        let Some(id_model_ident) = id_model_path.get_ident() else { abort!(id_model_path.span(), "First generic argument of `Id` type should be `Self`: `Id<Self, ..>`") };

        if id_model_ident.to_string() != "Self" {
            abort!(
                id_model_ident.span(),
                "First generic argument of `Id` type should be `Self`: `Id<Self, ..>`"
            )
        }

        self.validate_serde_id_attrs(meta_model.mongo.is_some());
    }

    fn get_id_type(&self) -> Ident {
        let ty = &self.ty;
        let path = match ty {
            Type::Path(TypePath { path, .. }) => path,
            _ => {
                abort!(ty.span(), "`id` field must be a path")
            }
        };
        let Some(last_segment) = path.segments.last() else { abort!(ty.span(), "Id type is empty") };
        if last_segment.ident.to_string() != "Id"
            && last_segment.ident.to_string() != "musty::prelude::Id"
        {
            abort!(ty.span(), "`id` field must be of type `musty::prelude::Id`")
        }

        let syn::PathArguments::AngleBracketed(path_arguments) = &last_segment.arguments else { abort!(ty.span(), "Id type must have generic arguments `<Self, YourIdType>` where `YourdIdType` implements `musty::prelude::IdGuard`") };

        let syn::GenericArgument::Type(syn::Type::Path(syn::TypePath { path: id_type_path, .. })) = path_arguments.args.last().unwrap() else { abort!(path_arguments.span(), "Last generic argument of `Id` type should be any type that implements `musty::prelude::IdGuard`: `Id<Self, YourIdType>`") };
        let Some(id_type_ident) = id_type_path.get_ident() else { abort!(id_type_path.span(), "Last generic argument of `Id` type should be an ident: `Id<Self, YourIdType>`") };
        id_type_ident.clone()
    }

    fn get_serde_attrs(&self) -> Vec<syn::Meta> {
        self.attrs
            .iter()
            .filter(|attr| attr.path.is_ident("serde"))
            .filter_map(|attr| match attr.parse_meta() {
                Ok(meta) => Some(meta),
                Err(err) => abort!(attr, "malformed attribute"; hint=err),
            })
            .map(|meta| match meta {
                syn::Meta::List(meta) => meta.nested,
                _ => abort!(
                    meta,
                    "expected this attribute to be formatted as a meta list: `#[serde(...)]`"
                ),
            })
            .fold(vec![], |mut acc, nested| {
                for inner in nested {
                    match inner {
                        syn::NestedMeta::Meta(meta) => acc.push(meta),
                        syn::NestedMeta::Lit(lit) => abort!(lit, "unexpected literal value"),
                    }
                }
                acc
            })
    }

    fn validate_serde_id_attrs(&self, mongo: bool) {
        let attrs = self.get_serde_attrs();
        let mut rename = false;
        let mut skip = false;
        for attr in &attrs {
            let attr_path = attr.path();
            if attr_path.is_ident("rename") {
                if mongo {
                    let serde_attr = SerdeFieldAttr::from_meta(attr).unwrap_or_else(
                        |err| abort!(attr, "Failed to parse serde attr `rename`"; hint=err),
                    );
                    if serde_attr.0 != "_id" {
                        abort!(attr, "The `rename` attribute for a MongoDB model must be `#[serde(rename = \"_id\")]`")
                    }
                    rename = true;
                }
            }
            if attr_path.is_ident("skip_serializing_if") {
                let serde_attr = SerdeFieldAttr::from_meta(attr).unwrap_or_else(|err| abort!(attr, "Failed to parse serde attr `skip_serializing_if`"; hint=err));
                if serde_attr.0 != "Id::is_none" && serde_attr.0 != "musty::prelude::Id::is_none" {
                    abort!(attr, "The `skip_serializing_if` attribute must be `#[serde(skip_serializing_if = \"Id::is_none\")]`")
                }
                skip = true;
            }
            if rename && skip {
                break;
            };
        }

        if !((rename || !mongo) && skip) {
            if mongo {
                abort!(self.ty.span(), "The `id` field must have the following serde attributes: `#[serde(rename = \"_id\", skip_serializing_if = \"Id::is_none\")]`")
            } else {
                abort!(self.ty.span(), "The `id` field must have the following serde attributes: `#[serde(skip_serializing_if = \"Id::is_none\")]`")
            }
        }
    }

    fn get_field_name(&self) -> Ident {
        if let Some(rename) = self.rename.as_ref() {
            Ident::new(rename, self.ident.as_ref().unwrap().span())
        } else {
            self.ident.clone().unwrap()
        }
    }
}

/// The root derive type for a model struct
#[derive(FromDeriveInput)]
#[darling(
    supports(struct_named),
    attributes(musty),
    forward_attrs(allow, doc, cfg)
)]
pub(crate) struct MetaModelDerive {
    pub(crate) vis: Visibility,
    pub(crate) ident: Ident,
    pub(crate) data: darling::ast::Data<darling::util::Ignored, MetaModelField>,
    pub(crate) mongo: Option<ModelMongoAttrs>,
    pub(crate) attrs: Vec<Attribute>,
}

impl MetaModelDerive {
    /// Get the type of the `id` field (or field with attribute #[musty(id)]) on the model struct
    fn get_model_id(&self) -> &MetaModelField {
        let ident = &self.ident;
        let data = &self.data;

        let fields = match data {
            darling::ast::Data::Struct(fields) => fields,
            _ => abort!(ident.span(), "Model must be a struct"),
        };

        let id_field = fields.iter().find(|field| field.is_id());

        if id_field.is_none() {
            abort!(ident.span(), "{} must have an `id` field", ident);
        }

        let id_field = id_field.unwrap();

        id_field.validate_id_field(&self);

        id_field
    }

    fn get_fields(&self) -> Vec<&MetaModelField> {
        let data = &self.data;

        let fields = match data {
            darling::ast::Data::Struct(fields) => fields,
            _ => abort!(self.ident.span(), "Model must be a struct"),
        };

        fields.iter().filter(|f| !f.skip).collect()
    }

    fn get_filter_struct_ident(&self) -> Ident {
        let ident = &self.ident;
        format_ident!("{}Filter", ident)
    }

    fn expand_filter_struct_function(&self, field: &MetaModelField) -> proc_macro2::TokenStream {
        let vis = &field.vis;
        let ident = field.get_field_name();
        let ty = &field.ty;
        let field_name = ident.to_string();
        if field.child {
            let child_ty = match ty {
                syn::Type::Path(syn::TypePath { path, .. }) => path,
                syn::Type::Reference(syn::TypeReference { elem, .. }) => match elem.as_ref() {
                    syn::Type::Path(syn::TypePath { path, .. }) => path,
                    _ => abort!(ident.span(), "Child field must be a path or ref"),
                },
                _ => abort!(ident.span(), "Child field must be a path or ref"),
            };
            let Some(child_ident) = child_ty.get_ident() else { abort!(ident.span(), "Type must be an in-scope ident") };
            let child_filter_ident = format_ident!("{}Filter", child_ident);
            return quote! {
                #vis fn #ident<Func>(&mut self, func: Func) -> &mut Self
                where
                    Func: FnOnce(&mut #child_filter_ident) -> &mut #child_filter_ident,
                {
                    self.child::<#child_filter_ident, Func>(#field_name, func);
                    self
                }
            };
        } else {
            let ty = if field.is_id() {
                let model_ident = &self.ident;
                let id_ty = field.get_id_type();
                quote! { musty::prelude::Id<#model_ident, #id_ty> }
            } else {
                quote! { #ty }
            };

            return quote! {
                #vis fn #ident(&mut self) -> musty::prelude::ModelFieldFilter<Self, #ty> {
                    self.field::<#ty>(#field_name)
                }
            };
        }
    }

    fn expand_filter_struct(&self) -> proc_macro2::TokenStream {
        let attrs = &self.attrs;
        let vis = &self.vis;
        let filter_struct_ident = self.get_filter_struct_ident();
        let fields = self.get_fields();
        let functions = fields
            .iter()
            .map(|f| Self::expand_filter_struct_function(&self, f))
            .collect::<Vec<_>>();

        quote! {
            #(
                #attrs
            )*
            #[derive(Debug, Clone)]
            #vis struct #filter_struct_ident {
                pub filter: musty::prelude::Filter,
            }

            #[automatically_derived]
            impl musty::prelude::ModelFilter for #filter_struct_ident {
                fn new() -> Self {
                    Self {
                        filter: musty::prelude::Filter::new(),
                    }
                }

                fn get_filter(&self) -> &musty::prelude::Filter {
                    &self.filter
                }

                fn get_filter_mut(&mut self) -> &mut musty::prelude::Filter {
                    &mut self.filter
                }
            }

            #[automatically_derived]
            impl #filter_struct_ident {
                #(
                    #functions
                )*
            }
        }
    }

    fn expand_filter_impls(&self) -> proc_macro2::TokenStream {
        let model_ident = &self.ident;
        let filter_struct_ident = self.get_filter_struct_ident();
        quote! {
            #[automatically_derived]
            impl musty::prelude::Filterable for #model_ident {
                type ModelFilter = #filter_struct_ident;
            }
        }
    }

    fn expand_model(&self) -> proc_macro2::TokenStream {
        let ident = &self.ident;

        let id_field = self.get_model_id();
        let model_id_type = id_field.get_id_type();

        quote! {
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
        }
    }

    pub(crate) fn expand_filter(&self) -> proc_macro2::TokenStream {
        let filter_struct = self.expand_filter_struct();
        let filter_impls = self.expand_filter_impls();
        quote! {
            #filter_struct
            #filter_impls
        }
    }

    /// Expands the model struct and the `Model` trait implementation for the model struct
    /// If the `mongo` attribute is present, this also expands the `MongoModel` trait implementation
    pub(crate) fn expand(self) -> proc_macro::TokenStream {
        let model_impl = (&self).expand_model();

        let filter = (&self).expand_filter();

        let mut expanded = quote! {
            #model_impl
            #filter
        };

        if let Some(mongo_attrs) = self.mongo.as_ref() {
            let mongo_model = super::mongo_model::expand_mongo_model(&self, &mongo_attrs);

            let mongo_fields_impl = super::mongo_model::expand_mongo_fields_impl(&self);

            expanded = quote! {
                #expanded

                #mongo_model

                #mongo_fields_impl
            };
        }

        expanded.into()
    }
}
