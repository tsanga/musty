use bson::{doc, oid::ObjectId};
use futures::StreamExt;
use mongodb::{options::ClientOptions, Client};
use musty::prelude::*;

#[model(mongo(collection = "users_find_many"))]
struct User {
    id: ObjectId,
    name: String,
}

#[tokio::main]
pub async fn main() -> musty::Result<()> {
    let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
    let client = Client::with_options(client_options)?;
    let db = client.database("musty").into();

    // Insert some users into the collection
    let mut user_jonah = User {
        id: ObjectId::new().into(),
        name: String::from("jonah"),
    };
    user_jonah.save(&db).await?;

    let mut user_alex = User {
        id: ObjectId::new().into(),
        name: String::from("alex"),
    };
    user_alex.save(&db).await?;

    // Get all users from the collection
    let mut cursor = User::find(&db, None, None).await?;
    while let Some(user) = cursor.next().await {
        println!("{:#?}", user?);
    }

    Ok(())
}
