use serde::{de::DeserializeOwned, ser::Serialize};
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::{Credentials, Signin};
use surrealdb::Surreal;

use yew::{hook,use_state, use_effect_with_deps};
use yew::{use_callback, Callback};

mod use_query_state;
mod use_self_ref;
mod use_surreal;

pub use use_query_state::*;
pub use use_self_ref::*;
pub use use_surreal::*;

use crate::logging::handle_error;
use crate::SurrealProps;
use crate::props::id::HasID;

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

/// Updates the local and remote data of this component with the Properties returned by the closure
#[hook]
pub fn use_update_callback<Props, IN, D, F>(
    f: F,
    deps: D,
) -> Callback<IN>
where
    Props: SurrealProps + HasID + PartialEq + Clone + 'static,
    <Props as SurrealProps>::Remote: PartialEq + Clone + Send + Sync + Serialize + DeserializeOwned,
    SurrealSelfRef<Props>: Clone,
    IN: 'static,
    F: Fn(IN, &D) -> Props + 'static,
    D: PartialEq + Clone + 'static,
    IN: 'hook,
    F: 'hook,
    D: 'hook,
{
    let sur = use_surreal();
    let r = use_self_ref::<Props>();
    use_callback(move |inp, (r, deps)| {
        let data = f(inp, deps);
        sur.update(r).with(data);
    }, (r, deps))
}
