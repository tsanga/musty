use musty::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[musty(mongo(collection = "users"))]
struct User {
    #[serde(rename = "_id", skip_serializing_if = "Id::is_none")]
    id: Id<Self, u32>,
    name: String,
    aliases: Vec<String>,
    #[musty(child)]
    address: Address,
}

#[derive(Debug, Clone, Serialize, Deserialize, Filter)]
struct Address {
    country: String,
    labels: Vec<String>,
}

#[tokio::main]
async fn main() -> musty::Result<()> {
    let filter = User::filter()
        .any(|user| {
            user.name()
                .any(|a| a.entry("jonah".to_string()).entry("alex".to_string()))
                .id()
                .eq(1.into())
        })
        .address(|addr| {
            addr.country()
                .any(|country| country.entry("US".to_string()).entry("CA".to_string()))
                .labels()
                .contains(|f| f.entry("test".to_string()))
        })
        .build();
    println!("filter: {:#?}", &filter);

    /*
    {
        "$or": [
            "name": { $or: [ "jonah", "alex" ] },
            "_id": 1
        ],
        "address": {
            "country": { $or: [ "US", "CA" ] }
        }
    }
    */

    Ok(())
}
