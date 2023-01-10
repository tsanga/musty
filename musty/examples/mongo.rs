use bson::doc;
use mongodb::{options::ClientOptions, Client};
use musty::prelude::*;

#[model(mongo(collection = "users"))]
struct User {
    id: u32,
    name: String,
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
    let client = Client::with_options(client_options)?;
    let db = Musty::mongo(client.database("musty"));

    let mut user = User {
        id: 0.into(),
        name: String::from("jonah"),
    };
    user.save(&db).await?;

    let user = User::find_one(&db, doc! { "name": "jonah" }, None).await?;
    println!("{:#?}", user);

    // alternatively:
    // let user = User::get(&musty, &id).await?;

    Ok(())
}
