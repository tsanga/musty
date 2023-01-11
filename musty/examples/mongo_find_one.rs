use mongodb::{options::ClientOptions, Client};
use bson::{oid::ObjectId, doc};
use musty::prelude::*;

#[model(mongo())] // The `collection = "name"` attribute is optional.  It will default to the name of your struct, converted to table case and plural (in this case: "users")
struct User {
    id: ObjectId,
    name: String,
}

#[tokio::main]
pub async fn main() -> musty::Result<()> {
    let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
    let client = Client::with_options(client_options)?;
    let db = Musty::mongo(client.database("musty"));

    // Insert a user into the collection
    let mut user = User { id: ObjectId::new().into(), name: String::from("jonah") };
    user.save(&db).await?;

    // Get the user from the collection by name
    let user = User::find_one(&db, doc! { "name": "jonah" }, None).await?;
    println!("{:#?}", user);

    Ok(())
}
