use serde::{de::DeserializeOwned, Serialize};
use yew::{
    function_component, html, BaseComponent, ContextProvider, Html, HtmlResult, Properties,
    Suspense,
};

use crate::{
    hooks::{use_query_state, SurrealSelfRef},
    props::{
        id::HasID,
        surreal_props::{PropsNoState, PropsWithState, SurrealProps},
    },
};

#[function_component]
pub fn Query<Inner>(props: &<<Inner as BaseComponent>::Properties as SurrealProps>::Local) -> Html
where
    Inner: BaseComponent,
    <Inner as BaseComponent>::Properties: SurrealProps + Clone + HasID,
    <<Inner as BaseComponent>::Properties as SurrealProps>::Remote:
        Send + Sync + DeserializeOwned + Serialize + Clone + PartialEq + HasID,
    <<Inner as BaseComponent>::Properties as SurrealProps>::Local: Properties + Clone,
    <<Inner as BaseComponent>::Properties as SurrealProps>::LocalWithState: Properties + Clone,
{
    let state = use_query_state::<Inner::Properties>(props.get_selector());
    let props_with_state = props.with_state(state);

    html!(
        <QueryWithState<Inner>  ..props_with_state/>
    )
}

/// perform the specified query and try to deserialize the answer as Inner::Properties
/// Then draw an Inner for each returned value
#[function_component]
pub fn QueryWithState<Inner>(
    local_props: &<<Inner as BaseComponent>::Properties as SurrealProps>::LocalWithState,
) -> Html
where
    Inner: BaseComponent,
    <Inner as BaseComponent>::Properties: SurrealProps + Clone + HasID,
    <<Inner as BaseComponent>::Properties as SurrealProps>::Remote:
        Send + Sync + DeserializeOwned + Serialize + Clone + PartialEq + HasID,
    <<Inner as BaseComponent>::Properties as SurrealProps>::LocalWithState: Properties + Clone,
{
    html!(
        <Suspense fallback={local_props.get_fallback().unwrap_or_default()}>
            <SuspendedQuery<Inner> ..{local_props.clone()}/>
        </Suspense>
    )
}

#[function_component(SuspendedQuery)]
pub fn suspended_query<Inner>(
    local_props: &<<Inner as BaseComponent>::Properties as SurrealProps>::LocalWithState,
) -> HtmlResult
where
    Inner: BaseComponent,
    <Inner as BaseComponent>::Properties: SurrealProps + Clone + HasID,
    <<Inner as BaseComponent>::Properties as SurrealProps>::Remote:
        Send + Sync + DeserializeOwned + Serialize + Clone + PartialEq + HasID,
    <<Inner as BaseComponent>::Properties as SurrealProps>::LocalWithState: Properties + Clone,
{
    let state = local_props.get_state();
    let data = state.get_data()?;
    Ok(html! {
        <>
            {for data.iter()
                .enumerate()
                .map(|(index, remote_props)| (index, Inner::Properties::construct(remote_props.clone(), local_props.clone())))
                .filter(|(_, props)| local_props.get_filter().as_ref().map(|x| x.emit(props.clone())).unwrap_or(true))
                .map(|(index, props)| {
                    let context = SurrealSelfRef {
                        state: state.clone(),
                        index,
                        id: props.id(),
                    };

                    html!(
                        <ContextProvider<SurrealSelfRef<Inner::Properties>> {context} key={props.id().to_string()}>
                            <Inner ..props.clone()/>
                        </ContextProvider<SurrealSelfRef<Inner::Properties>>>
                    )
            })}
        </>
    })
}
