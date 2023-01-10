use std::str::FromStr;

use bson::oid::ObjectId;
use mongodb::{options::ClientOptions, Client};
use musty::prelude::*;

#[derive(Model, Debug)]
struct User {
    id: Id<Self, ObjectId>,
}

#[tokio::main]
pub async fn main() -> Result<(), MustyError> {
    let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
    let client = Client::with_options(client_options)?;
    let musty = Musty::mongo(client.database("musty"));

    // save user
    let _user = User { id: Id::none() };

    // fetch a user by id
    let id: Id<User, ObjectId> = ObjectId::from_str("63bc2b0f4f603a0ab10e844d")?.into();
    match id.get_model(&musty).await {
        Ok(user) => println!("user: {:?}", user),
        Err(err) => println!("error: {:?}", err),
    }

    // alternatively:
    // let user = User::get(&musty, &id).await?;

    Ok(())
}
