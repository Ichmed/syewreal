use serde::{de::DeserializeOwned, ser::Serialize};
use surreal_macros::SELECT;
use surrealdb::sql::statements::SelectStatement;
use surrealdb::sql::{Query, Statement};
use yew::prelude::*;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::{Credentials, Signin};
use surrealdb::opt::IntoQuery;
use surrealdb::Surreal;
use yew::html::IntoPropValue;
use yew::{
    function_component, hook, html, use_effect_with_deps, use_state, AttrValue, BaseComponent,
    Html, Properties, UseStateHandle,
};

#[hook]
pub fn use_surreal_login<T>(
    client: &'static Surreal<Client>,
    url: String,
    login: impl Credentials<Signin, T> + 'static,
) -> SurrealToken
where
    surrealdb::method::Signin<'static, Client, T>:
        std::future::IntoFuture<Output = surrealdb::Result<T>>,
{
    let ready = use_state(|| false);
    let error = use_state(|| None);

    let ready_clone = ready.clone();
    let error_clone = error.clone();
    use_effect_with_deps(
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match client.connect::<Ws>(url).with_capacity(100000).await {
                    surrealdb::Result::Ok(_) => match client.signin(login).await {
                        surrealdb::Result::Ok(_) => { 
                            // TMP for development don't log in as root please
                            client.use_ns("test").use_db("test").await;
                            ready_clone.set(true);
                        }
                        surrealdb::Result::Err(err) => error_clone.set(Some(err)),
                    },
                    surrealdb::Result::Err(err) => error_clone.set(Some(err)),
                }
            });
        },
        (),
    );

    SurrealToken {
        client,
        ready,
        error,
    }
}

#[derive(Clone)]
pub struct SurrealToken {
    pub client: &'static Surreal<Client>,
    pub ready: UseStateHandle<bool>,
    pub error: UseStateHandle<Option<surrealdb::Error>>,
}

impl PartialEq for SurrealToken {
    fn eq(&self, other: &Self) -> bool {
        self.ready == other.ready
    }
}

impl SurrealToken {
    pub async fn do_query<T: DeserializeOwned>(
        &self,
        query: impl IntoQuery,
    ) -> surrealdb::Result<Vec<T>> {
        self.client.query(query).await?.take(0)
    }
}

#[hook]
pub fn use_surreal_select<T>(
    token: SurrealToken,
    selector: SelectStatement,
    parameters: Vec<(String, String)>,
) -> UseStateHandle<Vec<T>>
where
    T: 'static + Send + Sync + DeserializeOwned + Serialize,
{
    let state = use_state(|| Vec::new());
    let state_clone = state.clone();

    let SurrealToken {
        client,
        ready,
        error,
    } = token;

    let query = selector.into_query().expect("Statement can always be turned into queries");

    use_effect_with_deps(
        move |is_ready| {
            if **is_ready && error.is_none() {
                wasm_bindgen_futures::spawn_local(async move {
                    let mut q = client.query(query.clone());
                    for param in parameters {
                        q = q.bind(param);
                    }

                    match q.await {
                        Ok(mut r) => {
                            let list: surrealdb::Result<Vec<T>> = r.take(0);
                            match list {
                                Ok(result) => {
                                    web_sys::console::log_1(
                                        &format!("fetched {} items", result.len()).into(),
                                    );
                                    state_clone.set(result);
                                }
                                Err(error) => {
                                    format_error(error, query);
                                }
                            }
                        }
                        Err(error) => format_error(error, query),
                    }
                });
            }
        },
        ready,
    );

    state
}

fn format_error<Error: core::fmt::Debug>(error: Error, query: impl IntoQuery) {
    web_sys::console::error_1(
        &format!(
            "Error \"{:?}\"\nwhile performing query \"{:?}\"",
            error,
            query.into_query()
        )
        .into(),
    );
}

const NO_TOKEN_ERROR: &str = "No surreal token was set, either wrap this component or one of its parents into a <SurrealContext/>";

fn unwrap_token(token: Option<SurrealToken>) -> Result<SurrealToken, Html> {    
    match token {
        Some(token) => Ok(token.clone()),
        None => Err(html!(
            <div class="error surreal-error token-error">
                {NO_TOKEN_ERROR}
            </div>
        ))
    }
}

fn construct_selector(selector: Selector) -> Result<SelectStatement, Html> {
    match selector.base {
        Some(x) => Ok(x.clone()),
        None => Err(html!(
            <div class="error surreal-error query-error">{"Empty selector"}</div>
        ))
    }
}

/// perform the specified query and try to deserialize the answer as Inner::Properties
/// Then draw an Inner for the first value returned
#[function_component]
pub fn SurrealComponent<Inner>(
    props: &<<Inner as BaseComponent>::Properties as SurrealProp>::Local,
) -> Html
where
    Inner: BaseComponent,
    <Inner as BaseComponent>::Properties: PartialEq + SurrealProp + 'static + Clone,
    <<Inner as BaseComponent>::Properties as SurrealProp>::Remote:
        Send + Sync + DeserializeOwned + Serialize + Clone,
    <<Inner as BaseComponent>::Properties as SurrealProp>::Local:
        Properties + SurrealLocalProp + Clone,
{
    let token = match unwrap_token(use_context::<SurrealToken>()) {
        Ok(x) => x,
        Err(html) => return html
    };

    let mut query = match construct_selector(props.get_selector()) {
        Ok(x) => x,
        Err(html) => return html
    };

    query.limit = Some(surrealdb::sql::Limit(1.into()));

    let inner_props = use_surreal_select::<<Inner::Properties as SurrealProp>::Remote>(
        token,
        query,
        props.get_parameters()
            .params.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    );

    html! {for inner_props.first()
        .map(|remote| Inner::Properties::construct(
            remote.clone(),
            props.clone()
        ))
        .map(|prop| html!{<Inner ..prop/>})
    }
}


/// perform the specified query and try to deserialize the answer as Inner::Properties
/// Then draw an Inner for each returned value
#[function_component]
pub fn SurrealList<Inner>(
    props: &<<Inner as BaseComponent>::Properties as SurrealProp>::Local,
) -> Html
where
    Inner: BaseComponent,
    <Inner as BaseComponent>::Properties: PartialEq + SurrealProp + 'static + Clone,
    <<Inner as BaseComponent>::Properties as SurrealProp>::Remote:
        Send + Sync + DeserializeOwned + Serialize + Clone,
    <<Inner as BaseComponent>::Properties as SurrealProp>::Local:
        Properties + SurrealLocalProp + Clone,
{
    let token = match unwrap_token(use_context::<SurrealToken>()) {
        Ok(token) => token,
        Err(html) => return html
    };
    
    let query = match construct_selector(props.get_selector()) {
        Ok(x) => x,
        Err(html) => return html
    };

    let inner_props = use_surreal_select::<<Inner::Properties as SurrealProp>::Remote>(
        token,
        query,
        props.get_parameters()
            .params.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    );

    html! {for inner_props.iter()
        .map(|remote| Inner::Properties::construct(
            remote.clone(),
            props.clone()
        ))
        .map(|prop| match prop.get_id() {
            Some(id) => html!{<Inner key={id.to_string()} ..prop/>},
            None => html!{<Inner ..prop/>}
        })
    }
}

pub trait SurrealProp: Properties {
    type Remote;
    type Local;

    fn construct(remote: Self::Remote, local: Self::Local) -> Self;
    fn get_id(&self) -> Option<AttrValue>;
}

pub trait SurrealLocalProp {
    // fn get_token(&self) -> Option<SurrealToken>;
    fn get_selector(&self) -> Selector;
    fn get_parameters(&self) -> Parameters;
}

#[derive(Clone, PartialEq)]
pub struct Selector {
    base: Option<SelectStatement>,
}

impl From<Vec<Statement>> for Selector {
    fn from(value: Vec<Statement>) -> Self {
        Self {
            base: match value.first() {
                Some(Statement::Select(x)) => Some(x.clone()),
                _ => None
            }
        }
    }
}


impl IntoQuery for Selector {
    fn into_query(self) -> surrealdb::Result<Vec<surrealdb::sql::Statement>> {
        match self.base {
            Some(x) => x.into_query(),
            None => Ok(vec![])
        }
    }
}
impl IntoPropValue<Selector> for Query {
    fn into_prop_value(self) -> Selector {
        self.0.to_vec().into()
    }
}

impl IntoPropValue<Selector> for &str {
    fn into_prop_value(self) -> Selector {
        self.into_query().unwrap().into()
    }
}

impl IntoPropValue<Selector> for String {
    fn into_prop_value(self) -> Selector {
        self.into_query().unwrap().into()
    }
}

impl IntoPropValue<Selector> for surrealdb::Result<Vec<surrealdb::sql::Statement>> {
    fn into_prop_value(self) -> Selector {
        self.unwrap().into()
    }
}

impl IntoPropValue<Selector> for Vec<surrealdb::sql::Statement> {
    fn into_prop_value(self) -> Selector {
        self.into()
    }
}

#[derive(Clone, PartialEq)]
pub struct Parameters {
    params: Vec<(AttrValue, AttrValue)>,
}

impl Default for Parameters {
    fn default() -> Self {
        Parameters { params: vec![] }
    }
}

impl IntoPropValue<Parameters> for (String, String) {
    fn into_prop_value(self) -> Parameters {
        Parameters {
            params: vec![(self.0.into(), self.1.into())],
        }
    }
}

impl IntoPropValue<Parameters> for (&str, &str) {
    fn into_prop_value(self) -> Parameters {
        Parameters {
            params: vec![(self.0.to_owned().into(), self.1.to_owned().into())],
        }
    }
}

impl IntoPropValue<Parameters> for (&str, AttrValue) {
    fn into_prop_value(self) -> Parameters {
        Parameters {
            params: vec![(self.0.to_owned().into(), self.1.to_string().into())],
        }
    }
}


#[derive(Properties, PartialEq)]
pub struct SurrealContextProps {
    pub token: SurrealToken,
    pub children: Children,
}

#[function_component(SurrealContext)]
pub fn surreal_context(props: &SurrealContextProps) -> Html {
    
    if *props.token.ready {            
        html! {
            <ContextProvider<SurrealToken> context={props.token.clone()}>
                { for props.children.iter() }
            </ContextProvider<SurrealToken>>
        }
    }
    else {
        html! {
        }
    }
    
}
