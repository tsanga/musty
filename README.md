# musty

**An ODM for your favourite NoSQL database.**

**musty** is an asynchronous [object-document mapper](https://en.wikipedia.org/wiki/Object–relational_mapping) library for Rust. It turns your `struct`'s into queryable database models.

## example

```rust
use mongodb::{options::ClientOptions, Client};
use bson::{oid::ObjectId, doc};
use musty::prelude::*;

#[model(mongo(collection = "users"))]
struct User {
    id: ObjectId,
    name: String,
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let client_options = ClientOptions::parse("mongodb://localhost:27017").await?;
    let client = Client::with_options(client_options)?;
    let db = Musty::new(client.database("musty"));

    let mut user = User { id: ObjectId::new().into(), name: String::from("jonah") };
    user.save(&db).await?;

    let mut cursor = User::find(&db, None, None).await?;
    while let Some(user) = cursor.next().await {
        println!("{:?}", user?);
    }

    Ok(())
}


```