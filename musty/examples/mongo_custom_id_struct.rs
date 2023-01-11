use std::fmt::Display;

use mongodb::{options::ClientOptions, Client};
use bson::{doc, Bson};
use musty::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MyId(pub String);

impl Display for MyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<MyId> for Bson {
    fn from(id: MyId) -> Self {
        Bson::String(id.0)
    }
}

#[model(mongo(collection = "users_id_struct"))]
struct User {
    id: MyId,
    name: String,
}

#[tokio::main]
pub async fn main() -> musty::Result<()> {
    let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
    let client = Client::with_options(client_options)?;
    let db = Musty::mongo(client.database("musty"));

    // Insert a user into the collection
    let mut user = User { id: MyId("hello".to_string()).into(), name: String::from("jonah") };
    user.save(&db).await?;

    // Get the user from the collection by id
    let user = User::get_by_id(&db, MyId("hello".to_string())).await?;
    println!("{:#?}", user);

    Ok(())
}
