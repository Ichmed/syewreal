use std::ops::Deref;

use serde::de::DeserializeOwned;
// use yew::hook;
use yew::UseStateHandle;
use yew::html::IntoPropValue;
use yew::suspense::SuspensionResult;
use yew::use_state_eq;

use crate::logging;
use crate::props::selector::Selector;
use crate::props::surreal_props::SurrealProps;
use crate::hooks::use_surreal;

// Recursive expansion of hook! macro
// ===================================

#[cfg(not(doctest))]
#[doc = "\n# Note\n\nWhen used in function components and hooks, this hook is equivalent to:\n\n```\npub fn use_query_state<Props>(\n    selector: impl IntoPropValue<Selector>,\n) -> QueryState<Props::Remote>\nwhere\n    Props: SurrealProps,\n    Props::Remote: 'static + PartialEq + DeserializeOwned,\n{\n    /* implementation omitted */\n}\n\n```\n"]
pub fn use_query_state<'hook, 'arg0, Props>(
    selector: impl 'arg0 + IntoPropValue<Selector>,
) -> impl 'hook + ::yew::functional::Hook<Output = QueryState<Props::Remote>>
where
    Props: SurrealProps,
    Props::Remote: 'static + PartialEq + DeserializeOwned,
    'arg0: 'hook,
    Props: 'hook,
{
    use yew::{suspense::Suspension, use_effect_with_deps};

    fn inner_fn<'hook, 'arg0, Props>(
        _ctx: &mut ::yew::functional::HookContext,
        selector: impl 'arg0 + IntoPropValue<Selector>,
    ) -> QueryState<Props::Remote>
    where
        Props: SurrealProps,
        Props::Remote: 'static + PartialEq + DeserializeOwned,
        'arg0: 'hook,
        Props: 'hook,
    {
        let sur = ::yew::functional::Hook::run(use_surreal(), _ctx);
        let state: UseStateHandle<SuspensionResult<Vec<<Props as SurrealProps>::Remote>>> =
            ::yew::functional::Hook::run(use_state_eq(|| Err(Suspension::new().0)), _ctx);
        let selector = selector.into_prop_value();
        let state_inner = state.clone();
        {
            let selector = selector.clone();
            let state = state.clone();
            ::yew::functional::Hook::run(use_effect_with_deps(move |_| {
                state.set(Err(
                    sur.query(selector.clone())
                        .then(move |mut response| match response.take(0) {
                            Ok(data) => state_inner.set(Ok(data)),
                            Err(error) => logging::handle_error(error),
                })));
            }, ()), _ctx);
        }
        QueryState::<Props::Remote> { state: state, selector }
    }
    let boxed_inner = ::std::boxed::Box::new(
        move |_ctx: &mut ::yew::functional::HookContext| -> QueryState<Props::Remote> {
            inner_fn::<Props>(_ctx, selector)
        },
    )
        as ::std::boxed::Box<
            dyn 'hook
                + ::std::ops::FnOnce(&mut ::yew::functional::HookContext) -> QueryState<Props::Remote>,
        >;
    ::yew::functional::BoxedHook::<'hook, QueryState<Props::Remote>>::new(boxed_inner)
}
#[cfg(doctest)]
pub fn use_query_state<Props>(selector: impl IntoPropValue<Selector>) -> QueryState<Props::Remote>
where
    Props: SurrealProps,
    Props::Remote: 'static + PartialEq + DeserializeOwned,
{
    let sur = use_surreal();
    let state: UseStateHandle<Option<Vec<<Props as SurrealProps>::Remote>>> = use_state_eq(|| None);
    let selector = selector.into_prop_value();
    sur.query(selector.clone())
        .then(move |response| match response.take(0) {
            Ok(data) => state.set(Some(data)),
            Err(error) => logging::handle_error(error),
        });
    QueryState::<Props::Remote> { state, selector }
}

#[derive(Clone, PartialEq)]
pub struct QueryState<Remote> {
    state: UseStateHandle<SuspensionResult<Vec<Remote>>>,
    selector: Selector
}

impl<Remote> QueryState<Remote>
where
    Remote: Clone,
{
    pub fn get_data(&self) -> SuspensionResult<Vec<Remote>> {
        (*self.state).clone()
    }

    /// Return the internal data if any exists or an empty Vec<Remote> otherwise
    pub fn get_list(&self) -> Vec<Remote> {
        match &*self.state {
            Ok(data) => data.clone(),
            Err(_) => vec![],
        }
    }

    pub fn append(&self, data: Remote) {
        let mut existing = self.get_list();
        existing.push(data);
        self.state.set(Ok(existing));
    }

    pub fn set_target(&self, index: usize, data: Option<Remote>) {
        let existing = self.get_list();
        let mut capacity = existing.len();
        if data.is_some() {
            capacity += 1;
        }
        let mut new = Vec::with_capacity(capacity);
        new.extend_from_slice(&existing[..index]);
        if let Some(data) = data {
            new.push(data);
        }
        new.extend_from_slice(&existing[index+1..]);
        self.state.set(Ok(new));
    }

    pub fn get_selector(&self) -> Selector {
        self.selector.clone()
    }
}

impl<Remote> Deref for QueryState<Remote> {
    type Target = UseStateHandle<SuspensionResult<Vec<Remote>>>;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
