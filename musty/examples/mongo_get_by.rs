#![allow(unused_variables)]

use mongodb::{options::ClientOptions, Client};
use bson::{oid::ObjectId, doc};
use musty::prelude::*;

#[model(mongo(collection = "users"))] // The `collection = "name"` attribute is optional.  It will default to the name of your struct, converted to table case and plural (in this case: "users")
struct User {
    id: ObjectId,
    #[musty(get())] // generates a `User::get_by_name(db, name)` function
    name: String,
    #[musty(get(name = "get_from_email"))] // generates a `User::get_from_email(db, email)` function
    email: String,
}

#[tokio::main]
pub async fn main() -> musty::Result<()> {
    let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
    let client = Client::with_options(client_options)?;
    let db = Musty::mongo(client.database("musty"));

    // Insert a user into the collection
    let mut user = User { id: ObjectId::new().into(), name: String::from("jonah"), email: String::from("jonah@tsanga.net") };
    user.save(&db).await?;

    // Get the user from the collection by name, using the generated method
    let user_by_name = User::get_by_name(&db, "jonah".to_string()).await?;
    println!("user_by_name: {:#?}", user_by_name);

    let user_from_email = User::get_from_email(&db, "jonah@tsanga.net".to_string()).await?;
    println!("user_from_email: {:#?}", user_from_email);

    assert_eq!(user_by_name.unwrap().id, user_from_email.unwrap().id);

    Ok(())
}
