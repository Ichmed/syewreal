use serde::{de::DeserializeOwned, ser::Serialize};
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::method::Query;
use surrealdb::opt::auth::{Credentials, Signin};
use surrealdb::opt::IntoQuery;
use surrealdb::sql::statements::SelectStatement;
use surrealdb::{Connection, Surreal};

use yew::suspense::{Suspension, SuspensionResult};
use yew::use_context;
use yew::{hook, use_effect_with_deps, use_state, UseStateHandle};

mod use_query_state;
mod use_self_ref;
mod use_surreal;

pub use use_query_state::*;
pub use use_self_ref::*;
pub use use_surreal::*;

use crate::logging::handle_error;

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

    let ready_clone = ready.clone();
    use_effect_with_deps(
        move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match client.connect::<Ws>(url).with_capacity(100000).await {
                    surrealdb::Result::Ok(_) => match client.signin(login).await {
                        surrealdb::Result::Ok(_) => {
                            // TMP for development don't log in as root please
                            // client.use_ns("test").use_db("test").await;
                            ready_clone.set(true);
                        }
                        Err(error) => handle_error(error),
                    },
                    Err(error) => handle_error(error),
                }
            });
        },
        (),
    );

    SurrealToken { client, ready }
}

struct State<T>(bool, Option<Vec<T>>);

async fn fetch_select<T, C: std::fmt::Debug>(
    target: UseStateHandle<State<T>>,
    query: Query<'static, C>,
) where
    T: DeserializeOwned,
    C: 'static + Connection,
{
    match query.await {
        Ok(mut r) => {
            let list: surrealdb::Result<Vec<T>> = r.take(0);
            match list {
                Ok(result) => {
                    target.set(State(true, Some(result)));
                }
                Err(error) => {
                    format_error(error);
                }
            }
        }
        Err(error) => format_error(error),
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
    let state = use_state(|| State(false, None));
    let state_clone = state.clone();

    use_effect_with_deps(
        move |_| state_clone.set(State(true, None)),
        (selector.clone(), parameters.clone()),
    );

    let token = match use_context::<SurrealToken>() {
        Some(token) => token,
        None => {
            web_sys::console::error_1(
                &"Surreal Components must be wrappen <SurrealContext/>".into(),
            );
            return Err(Suspension::new().0);
        }
    };

    match *state {
        State(false, _) => Ok(vec![]),
        State(true, Some(ref result)) => Ok(result.to_vec()),
        State(true, None) => {
            let client = token.client;

            let select_query = selector
                .into_query()
                .expect("Statement can always be turned into queries");

            let mut q = client.query(select_query);
            for param in parameters {
                q = q.bind(param);
            }
            Err(Suspension::from_future(fetch_select(state, q)))
        }
    }
}

fn format_error<Error: core::fmt::Debug>(error: Error) {
    web_sys::console::error_1(&format!("Error \"{:?}\"\nwhile performing query", error,).into());
}
