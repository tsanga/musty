# musty

**An ODM for your favourite NoSQL database.**

## example

```rust
use bson::{oid::ObjectId};
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
    let db = Musty::mongo(client.database("musty"));

    let mut user = User { name: String::from("jonah") };
    user.save(&db).await?;

    let mut cursor = User::find(&db, None, None).await?;
    while let Some(user) = cursor.next().await {
        println!("{:?}", user?);
    }

    Ok(())
}


```