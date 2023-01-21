use musty::prelude::*;
use serde::{Deserialize, Serialize};

#[model(mongo(collection = "users"))]
#[derive(Filter)]
struct User {
    id: u32,
    name: String,
    aliases: Vec<String>,
    #[filter(child)]
    address: Address,
}

#[derive(Debug, Filter, Serialize, Deserialize)]
struct Address {
    country: String,
    labels: Vec<String>,
}

#[tokio::main]
async fn main() -> musty::Result<()> {
    let filter = User::filter()
        .id()
        .eq(1.into())
        .address(|address| {
            address
                .country()
                .any(|f| f.entry("USA".to_string()).entry("CA".to_string()))
        })
        .build();

    println!("filter: {:#?}", &filter);
    Ok(())
}

/*pub enum Ref<T: Model> {
    Id(Id<T>),
    Model(T),
}*/
