use yew::{Properties, Callback, Html};

use crate::hooks::QueryState;

use super::selector::{Selector, Parameters};


pub trait SurrealProps: Properties + Sized{
    type Remote;
    type Local: PropsNoState<Self, Self::Remote, Self::LocalWithState>;
    type LocalWithState: PropsWithState<Self, Self::Remote>;

    fn construct(remote: Self::Remote, local: Self::LocalWithState) -> Self;
    fn get_remote(&self) -> Self::Remote;
}

pub trait PropsNoState<Full, Remote, WithState: PropsWithState<Full, Remote>> {
    fn with_state(&self, state: QueryState<Remote>) -> WithState;
    fn get_selector(&self) -> Selector;
}

pub trait PropsWithState<Full, Remote> {
    fn get_parameters(&self) -> Parameters;
    fn get_state(&self) -> QueryState<Remote>;
    fn get_filter(&self) -> Option<Callback<Full, bool>>;
    fn get_fallback(&self) -> Option<Html> {
        None
    }
}