use darling::{FromDeriveInput, FromMeta, FromField};
use proc_macro::{TokenStream};
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::{DeriveInput, Ident, TypePath, Type, Path};
use proc_macro_error::{
    abort
};

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
    collection_name: Option<String>,
}

#[derive(FromDeriveInput)]
#[darling(attributes(model), forward_attrs(allow, doc, cfg))]
pub(crate) struct MetaModelDerive {
    ident: Ident,
    data: darling::ast::Data<darling::util::Ignored, MetaModelField>,
    mongo: Option<MongoAttrs>,
}

impl MetaModelDerive {
    fn get_model_id_type(&self) -> String {
        let ident = &self.ident;
        let data = &self.data;

        let fields = match data {
            darling::ast::Data::Struct(fields) => fields,
            _ => abort!(ident.span(), "Model must be a struct")
        };

        let id_field = fields
            .iter()
            .find(|field| field.ident == Some(Ident::new("id", Span::call_site())));

        if id_field.is_none() {
            abort!(ident.span(), "{} must have an `id` field", ident);
        }

        let path = match &id_field.unwrap().ty {
            Type::Path(TypePath { path, .. }) => path,
            _ => { abort!(ident.span(), "{} `id` field must be an Id", ident) },
        };

        let mut segments = path.segments.iter();
        let id_type = segments.next().unwrap().ident.to_string();

        abort!(ident.span(), "{}", id_type);
    }

    pub fn expand(&self) -> proc_macro::TokenStream {
        let ident = &self.ident;

        let model_id_type = self.get_model_id_type();

        quote! {
            use musty::model::{Model};

            impl Model for #ident {

            }
        }.into()
    }
}
