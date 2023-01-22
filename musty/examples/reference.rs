use mongodb::{options::ClientOptions, Client};
use musty::prelude::*;

#[model(mongo(collection = "ref_users"))]
#[derive(Debug, Filter)]
struct User {
    id: u32,
    name: String,
    address: Ref<Address>,
}

#[model(mongo(collection = "ref_user_addresses"))]
#[derive(Debug, Filter)]
struct Address {
    id: u32,
    street: String,
    city: String,
    country: String,
}

#[tokio::main]
async fn main() -> musty::Result<()> {
    let db = Musty::new(Client::with_options(ClientOptions::parse("mongodb://localhost:27017").await?)?.database("musty"));

    // Insert an address into the collection
    let mut address = Address {
        id: 1.into(),
        street: String::from("123 Main St"),
        city: String::from("New York"),
        country: String::from("USA"),
    };
    address.save(&db).await?;

    // Insert a user into the collection
    let mut user = User {
        id: 1.into(),
        name: String::from("jonah"),
        address: Ref::new(1.into()),
    };
    user.save(&db).await?;

    // Get the user from the collection by id
    let user = User::get_by_id(&db, 1).await?.expect("User not found");
    println!("User: {:#?}", user);

    // Get the user's address by the `address` Ref in the user object
    // If the address was populated when the User was queried, the Ref<Address> will be Ref::Model(address), 
    // and will not make a database call
    let address = user.address.get(&db).await?.expect("Address not found");
    println!("Address: {:#?}", address);

    Ok(())
}