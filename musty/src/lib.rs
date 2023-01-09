mod id;
mod model;
mod cursor;
mod error;
mod backend;
mod db;

pub use id::Id;
pub use model::Model;

use musty_macro::Model as MustyModel;
pub mod prelude {
    // TODO
}


#[derive(MustyModel)]
struct User {
    id: Id<Self>,
}