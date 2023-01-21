use darling::{FromDeriveInput, FromField};
use proc_macro2::Ident;
use quote::{quote, format_ident};
use proc_macro_error::abort;
use syn::{Visibility, Attribute};

/// A field on a model struct
#[derive(FromField)]
#[darling(attributes(filter), forward_attrs(doc, cfg, allow))]
pub(crate) struct MetaFilterField {
    pub(crate) vis: syn::Visibility,
    pub(crate) ident: Option<Ident>,
    pub(crate) ty: syn::Type,
    /// rename a field: #[musty(rename = "new_field_name")]
    #[darling(default)]
    pub(crate) rename: Option<String>,
    /// if this field is a nested/child object (document), add the #[musty(child)] attribute to the field
    /// to generate filter functions for the nested/child object
    /// requires that the nested object type implements `musty::prelude::ModelFilter` via `#[derive(Filter)]`
    #[darling(default)]
    pub(crate) child: bool,
    pub(crate) attrs: Vec<Attribute>,
}

impl MetaFilterField {
    pub fn get_field_name(&self) -> Ident {
        if let Some(rename) = self.rename.as_ref() {
            Ident::new(rename, self.ident.as_ref().unwrap().span())
        } else {
            self.ident.clone().unwrap()
        }
    }
}

#[derive(FromDeriveInput)]
#[darling(supports(struct_named), forward_attrs(allow, cfg))]
pub(crate) struct MetaFilter {
    pub(crate) ident: Ident,
    pub(crate) vis: Visibility,
    pub(crate) data: darling::ast::Data<darling::util::Ignored, MetaFilterField>,
    pub(crate) attrs: Vec<Attribute>,
}

impl MetaFilter {
    pub(crate) fn get_fields(&self) -> Vec<&MetaFilterField> {
        let data = &self.data;

        let fields = match data {
            darling::ast::Data::Struct(fields) => fields,
            _ => abort!(self.ident.span(), "Model must be a struct"),
        };

        fields.iter().collect()
    }

    fn get_filter_struct_ident(&self) -> Ident {
        let ident = &self.ident;
        format_ident!("{}Filter", ident)
    }

    fn expand_filter_struct_function(field: &MetaFilterField) -> proc_macro2::TokenStream {
        let vis = &field.vis;
        let ident = field.get_field_name();
        let ty = &field.ty;
        let field_name = ident.to_string();
        let attrs = &field.attrs;
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
                #(
                    #attrs
                )*
                #vis fn #ident<Func>(&mut self, func: Func) -> &mut Self
                where
                    Func: FnOnce(&mut #child_filter_ident) -> &mut #child_filter_ident,
                {
                    self.child::<#child_filter_ident, Func>(#field_name, func);
                    self
                }
            }
        } else {
            return quote! {
                #(
                    #attrs
                )*
                #vis fn #ident(&mut self) -> musty::prelude::ModelFieldFilter<Self, #ty> {
                    self.field::<#ty>(#field_name)
                }
            }
        }
    }

    fn expand_filter_struct(&self) -> proc_macro2::TokenStream {
        let attrs = &self.attrs;
        let vis = &self.vis;
        let filter_struct_ident = self.get_filter_struct_ident();
        let fields = self.get_fields();
        let functions = fields.iter().map(|f| Self::expand_filter_struct_function(f)).collect::<Vec<_>>();

        quote! {
            #(
                #attrs
            )*
            #[derive(Debug, Clone)]
            #vis struct #filter_struct_ident {
                pub filter: musty::prelude::Filter,
            }

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
            impl musty::prelude::Filterable for #model_ident {
                type ModelFilter = #filter_struct_ident;
            }
        }
    }

    pub(crate) fn expand(&self) -> proc_macro::TokenStream {
        let filter_struct = self.expand_filter_struct();
        let filter_impls = self.expand_filter_impls();

        quote! {
            #filter_struct
            #filter_impls
        }.into()
    }
}