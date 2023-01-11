pub mod surreal;
pub mod hooks;
pub use surreal_macros::*;

pub type Client = surrealdb::Surreal<surrealdb::engine::remote::ws::Client>;
pub type Login<'a> = surrealdb::opt::auth::Database<'a>;
