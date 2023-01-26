
use std::error::Error;
use std::fmt::Display;

use serde::{Serialize, de::DeserializeOwned};
use yew::{use_context, hook};

use crate::logging;
use crate::props::id::ID;
use crate::props::surreal_props::SurrealProps;

use super::QueryState;

#[derive(Debug)]
struct NoSelfRef;

impl Display for NoSelfRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Components must be managed by a <Query/> or <QueryWithState/> component to use self_ref")
    }
}

impl Error for NoSelfRef {}

#[hook]
pub fn use_self_ref<T>() -> SurrealSelfRef<T>
where
    T: 'static + SurrealProps + Clone,
    T::Remote: PartialEq + Clone + Serialize + DeserializeOwned + 'static,
{
    match use_context::<SurrealSelfRef<T>>() {
        Some(hook) => hook,
        None => logging::panic_error(NoSelfRef)
    }
}

#[derive(Clone, PartialEq)]
pub struct SurrealSelfRef<T: SurrealProps> {
    pub(crate) state: QueryState<T::Remote>,
    pub(crate) index: usize,
    pub(crate) id: ID
}

impl<T: SurrealProps> SurrealSelfRef<T> 
where T::Remote: Clone
{
    pub fn set(&self, data: Option<T::Remote>) {
        self.state.set_target(self.index, data);
    }
}