use bson::doc;
use mongodb::{options::ClientOptions, Client};
use musty::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[musty(mongo(collection = "users"))]
struct User {
    #[serde(rename = "_id", skip_serializing_if = "Id::is_none")]
    id: Id<Self, u32>,
    name: String,
}

#[tokio::main]
pub async fn main() -> musty::Result<()> {
    let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
    let client = Client::with_options(client_options)?;
    let db = Musty::new(client.database("musty"));

    // Insert a user into the collection
    let mut user = User {
        id: 1.into(),
        name: String::from("jonah"),
    };
    user.save(&db).await?;

    // Get the user from the collection by id
    let user = User::get_by_id(&db, 1).await?;
    println!("{:#?}", user);

    Ok(())
}
