// pub mod surreal;
pub mod props;
pub mod hooks;
pub mod components;
pub use surreal_macros::*;

mod logging;

pub use props::surreal_props::SurrealProps;

pub type Client = surrealdb::Surreal<surrealdb::engine::remote::ws::Client>;
pub type Login<'a> = surrealdb::opt::auth::Database<'a>;
pub type RootLogin<'a> = surrealdb::opt::auth::Root<'a>;