use yew::{Properties, Html, Children, function_component, ContextProvider, Suspense, html};

use crate::hooks::SurrealToken;



#[derive(Properties, PartialEq)]
pub struct SurrealContextProps {
    pub token: SurrealToken,
    pub fallback: Option<Html>,
    pub children: Children,
}

#[function_component(SurrealContext)]
pub fn surreal_context(props: &SurrealContextProps) -> Html {
    let fallback = props.fallback.clone().unwrap_or(html!());

    if *props.token.ready {
        html! {
            <Suspense {fallback}>
                <ContextProvider<SurrealToken> context={props.token.clone()}>
                    { for props.children.iter() }
                </ContextProvider<SurrealToken>>
            </Suspense>
        }
    } else {
        fallback
    }
}