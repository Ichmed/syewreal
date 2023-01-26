use serde::{de::DeserializeOwned, Serialize};
use yew::{function_component, html, BaseComponent, ContextProvider, Html, Properties};

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
    let state = local_props.get_state();

    match state.get_data() {
        None => html!(),
        Some(data) => html! {
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
        },
    }
}

// #[derive(Properties, Clone)]
// pub(crate) struct ContainerProps<Inner>
// where
//     Inner: BaseComponent,
//     <Inner as BaseComponent>::Properties: PartialEq + SurrealProps,
//     <<Inner as BaseComponent>::Properties as SurrealProps>::Remote: Clone + PartialEq,
//     <<Inner as BaseComponent>::Properties as SurrealProps>::LocalWithState:
//         Properties + Clone + PartialEq,
// {
//     local: <<Inner as BaseComponent>::Properties as SurrealProps>::LocalWithState,
//     remote: <<Inner as BaseComponent>::Properties as SurrealProps>::Remote,
//     selector: Selector,
//     add_handle: Callback<<<Inner as BaseComponent>::Properties as SurrealProps>::Remote, ()>,
//     filter: Option<Callback<<<Inner as BaseComponent>::Properties as SurrealProps>::Remote, bool>>,
// }

// impl<Inner> PartialEq for ContainerProps<Inner>
// where
//     Inner: BaseComponent,
//     <Inner as BaseComponent>::Properties: PartialEq + SurrealProps,
//     <<Inner as BaseComponent>::Properties as SurrealProps>::Remote: Clone + PartialEq,
//     <<Inner as BaseComponent>::Properties as SurrealProps>::LocalWithState:
//         Properties + Clone + PartialEq,
// {
//     fn eq(&self, other: &Self) -> bool {
//         self.local == other.local && self.remote == other.remote && self.selector == other.selector
//     }
// }

// #[function_component(SurrealContainer)]
// pub(crate) fn surreal_container<Inner>(props: &ContainerProps<Inner>) -> Html
// where
//     Inner: BaseComponent,
//     <Inner as BaseComponent>::Properties: SurrealProps + Clone + HasID,
//     <<Inner as BaseComponent>::Properties as SurrealProps>::Remote:
//         Send + Sync + DeserializeOwned + Serialize + Clone + PartialEq,
//     <<Inner as BaseComponent>::Properties as SurrealProps>::LocalWithState: Properties + Clone,
// {
//     let internal_remote = use_state_eq(|| None);

//     {
//         let remote = internal_remote.clone();
//         use_effect_with_deps(
//             move |remote_data| {
//                 remote.set(Some((*remote_data).clone()));
//             },
//             props.remote.clone(),
//         );
//     }

//     match (*internal_remote).clone() {
//         Some(remote_data) => {
//             let inner_props =
//                 Inner::Properties::construct(remote_data.clone(), props.local.clone());

//             let context = SurrealSelfRef {
//                 state: internal_remote,
//                 selector: props.selector.clone(),
//                 id: inner_props.id(),
//             };

//             if props.filter.as_ref().map(|x| x.emit(remote_data)).unwrap_or(true) {
//                 html!(
//                     <ContextProvider<SurrealSelfRef<Inner::Properties>> {context}>
//                         <Inner ..inner_props/>
//                     </ContextProvider<SurrealSelfRef<Inner::Properties>>>
//                 )
//             } else {
//                 html!()
//             }

//         }
//         None => {
//             html!()
//         }
//     }
// }
