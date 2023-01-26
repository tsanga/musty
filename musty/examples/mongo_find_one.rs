#![allow(unused_variables)]

use bson::{doc, oid::ObjectId};
use mongodb::{options::ClientOptions, Client};
use musty::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[musty(mongo(collection = "users"))] // The `collection = "name"` attribute is optional.  It will default to the name of your struct, converted to table case and plural (in this case: "users")
struct User {
    #[serde(rename = "_id", skip_serializing_if = "Id::is_none")]
    id: Id<Self, ObjectId>,
    #[musty(mongo(get))] // generates a `User::get_by_name(db, name)` method
    name: String,
}

#[tokio::main]
pub async fn main() -> musty::Result<()> {
    let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
    let client = Client::with_options(client_options)?;
    let db = Musty::new(client.database("musty"));

    // Insert a user into the collection
    let mut user = User {
        id: ObjectId::new().into(),
        name: String::from("jonah"),
    };
    user.save(&db).await?;

    // Get the user from the collection by name
    let user = User::find_one(&db, doc! { "name": "jonah" }).await?;
    println!("{:#?}", user);

    let user2 = User::get_by_name(&db, "jonah".to_string()).await?;
    println!("{:#?}", user);

    Ok(())
}
