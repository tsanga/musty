use darling::{FromDeriveInput, FromField, FromMeta};

use crate::util::string::{ToPlural, ToTableCase};
use proc_macro2::Span;
use proc_macro_error::abort;
use quote::quote;
use syn::{Ident, Path, Type, TypePath};

/*

#[derive(Model)]
#[model(mongo(collection_name = "users"))]
struct User {
    pub id: Id<Self, ObjectId>,
    #[get_by]
    pub name: String,
}

// generates:

impl Model<ObjectId> for User {
    pub fn id(&self) -> &Id<Self, ObjectId> {
        &self.
    }

    pub fn set_id(&mut self, id: Id<Self, ObjectId>) {
        self.id = id;
    }
}

impl MongoModel for User {
    const COLLECTION_NAME: &'static str = "users";
}

impl User {
    pub async fn get_by_name(db: &Db, name: &str) -> Result<Self> {
        let collection = db.collection(Self::COLLECTION_NAME);
        let filter = doc! { "name": name };
        let user = collection.find_one(filter, None).await?;
        Ok(user)
    }
}

*/

#[derive(FromField)]
pub struct MetaModelField {
    ident: Option<Ident>,
    ty: syn::Type,
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
    data: darling::ast::Data<darling::util::Ignored, MetaModelField>,
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
            .find(|field| field.ident == Some(Ident::new("id", Span::call_site())));

        if id_field.is_none() {
            abort!(ident.span(), "{} must have an `id` field", ident);
        }

        let path = match &id_field.unwrap().ty {
            Type::Path(TypePath { path, .. }) => path,
            _ => {
                abort!(ident.span(), "{} `id` field must be an Id", ident)
            }
        };

        let segment = &path.segments.iter().next().unwrap();
        let arguments = &segment.arguments;

        if let syn::PathArguments::AngleBracketed(arguments) = arguments {
            let id_type = &arguments.args.iter().last().unwrap();

            if arguments.args.len() == 1 {
                return Path::from_string("musty::prelude::DefaultIdType").unwrap();
            }

            if let syn::GenericArgument::Type(Type::Path(TypePath { path, .. })) = id_type {
                let segment = &path.segments.iter().next().unwrap();
                return Path::from(segment.ident.clone());
            }
        } else {
            abort!(ident.span(), "{} `id` field must be an Id", ident);
        }

        abort!(ident.span(), "{}{:?}", quote! {#path}, segment);
    }

    pub fn expand(self) -> proc_macro::TokenStream {
        let ident = &self.ident;

        let model_id_type = self.get_model_id_type();

        let mut model = quote! {
            impl musty::prelude::Model<#model_id_type> for #ident where Self: Sized {
                fn id(&self) -> &Id<Self, #model_id_type> {
                    &self.id
                }

                fn set_id(&mut self, id: Id<Self, #model_id_type>) {
                    self.id = id;
                }
            }
        };

        if let Some(mongo_attrs) = self.mongo {
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
                impl musty::prelude::MongoModel<#model_id_type> for #ident where Self: Sized {
                    const COLLECTION_NAME: &'static str = #collection_name;
                }

                #[async_trait]
                impl musty::prelude::Identifable<#model_id_type> for musty::prelude::Id<#ident, #model_id_type> {
                    type Model = #ident;
                    type Database = mongodb::Database;

                    async fn get(self, db: &Self::Database) -> std::result::Result<Self::Model, musty::prelude::MustyError> {
                        panic!("not implemented")
                    }
                }
            };
        }

        quote! {
            #model
        }
        .into()
    }
}
