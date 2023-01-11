use crate::{surreal::SurrealToken};
use surrealdb::{Surreal, Connection};
use surrealdb::method::Query;
use surrealdb::opt::auth::{Credentials, Signin};
use surrealdb::opt::IntoQuery;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::sql::statements::SelectStatement;
use serde::{de::DeserializeOwned, ser::Serialize};

use yew::suspense::Suspension;
use yew::use_context;
use yew::{
    hook, use_effect_with_deps, use_state, UseStateHandle,
    suspense::SuspensionResult
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

async fn fetch_select<T, C>(target: UseStateHandle<Option<Vec<T>>>, query: Query<'static, C>) 
where 
    T: DeserializeOwned,
    C: 'static + Connection
{
    match query.await {
        Ok(mut r) => {
            let list: surrealdb::Result<Vec<T>> = r.take(0);
            match list {
                Ok(result) => {
                    web_sys::console::log_1(
                        &format!("fetched {} items", result.len()).into(),
                    );
                    target.set(Some(result));
                }
                Err(error) => {
                    // format_error(error, query.query);
                }
            }
        }
        Err(error) => () //format_error(error, query),
    }
}

#[hook]
pub fn use_surreal_select<T>(
    selector: SelectStatement,
    parameters: Vec<(String, String)>,
) -> SuspensionResult<Vec<T>>
where
    T: 'static + Send + Sync + DeserializeOwned + Serialize + Clone,
{
    let state = use_state(|| Option::<Vec<T>>::None);
    let state_clone = state.clone();

    use_effect_with_deps(move |_| state_clone.set(None), (selector.clone(), parameters.clone()));

    // let state_clone = state.clone();
    
    let token = match use_context::<SurrealToken>() {
        Some(token) => token,
        None => {
            web_sys::console::error_1(&"Surreal Components must be wrappen <SurrealContext/>".into());
            return Err(Suspension::new().0)
        }
    };

    match *state {
        Some(ref result) => Ok(result.to_vec()),
        None => {
            
            let client = token.client;
        
            let select_query = selector.into_query().expect("Statement can always be turned into queries");
        
            let mut q = client.query(select_query);
            for param in parameters {
                q = q.bind(param);
            }
            Err(Suspension::from_future(fetch_select(state, q)))
        }
    }

    


    // use_effect_with_deps(
    //     move |is_ready| {
    //         if **is_ready && error.is_none() {
    //             wasm_bindgen_futures::spawn_local(async move {
    //                 let mut q = client.query(query.clone());
    //                 for param in parameters {
    //                     q = q.bind(param);
    //                 }

    //                 match q.await {
    //                     Ok(mut r) => {
    //                         let list: surrealdb::Result<Vec<T>> = r.take(0);
    //                         match list {
    //                             Ok(result) => {
    //                                 web_sys::console::log_1(
    //                                     &format!("fetched {} items", result.len()).into(),
    //                                 );
    //                                 state_clone.set(result);
    //                             }
    //                             Err(error) => {
    //                                 format_error(error, query);
    //                             }
    //                         }
    //                     }
    //                     Err(error) => format_error(error, query),
    //                 }
    //             });
    //         }
    //     },
    //     ready,
    // );

    // state
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