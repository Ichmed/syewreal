pub mod surreal;
pub use surreal_macros::*;


pub use surrealdb::{
    Surreal, 
    engine::remote::ws::Client, 
    opt::auth::{
        Root,
        Database
    }, 
    sql::statements::SelectStatement};

