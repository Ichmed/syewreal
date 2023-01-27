use std::{fmt::Display, error::Error};

use serde::{de::DeserializeOwned, Serialize};
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    method::{Content, Query, Select},
    opt::{
        auth::{Credentials, Signin},
        IntoQuery, IntoResource,
    },
    sql::{statements::SelectStatement, Value, Values},
    Connection, Response, Result, Surreal,
};
use yew::{hook, use_context, UseStateHandle, suspense::Suspension};

use crate::{props::id::HasID, props::surreal_props::SurrealProps, logging::{handle_error, self}};

use super::{QueryState, SurrealSelfRef};

#[derive(Debug)]
struct NoSurrealContext;

impl Display for NoSurrealContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Surreal Components must be wrapped in a <SurrealContext/> component")
    }
}

impl Error for NoSurrealContext {}


#[hook]
pub fn use_surreal() -> SurrealToken {
    match use_context::<SurrealToken>() {
        Some(hook) => hook,
        None => logging::panic_error(NoSurrealContext),
    }
}

#[derive(Clone)]
pub struct SurrealToken {
    pub client: &'static Surreal<Client>,
    pub ready: UseStateHandle<bool>,
}

impl PartialEq for SurrealToken {
    fn eq(&self, other: &Self) -> bool {
        self.ready == other.ready
    }
}

impl SurrealToken {
    pub fn sign_in<R>(&self, url: String, credentials: impl 'static + Credentials<Signin, R>)
    where
        R: DeserializeOwned + Send + Sync,
    {
        let client = self.client;
        let ready = self.ready.clone();

        self.ready.set(false);

        Suspension::from_future(async move {
            match client.connect::<Ws>(url).with_capacity(100000).await {
                surrealdb::Result::Ok(()) => match client.signin(credentials).await {
                    surrealdb::Result::Ok(_) => {
                        ready.set(true);
                    }
                    Err(error) => handle_error(error),
                },
                Err(error) => handle_error(error),
            }
        });
    }

    pub fn select<R: DeserializeOwned>(
        &self,
        resource: impl IntoResource<R>,
    ) -> SurrealSelect<Client, R> {
        SurrealSelect(self.client.select(resource))
    }

    pub fn update<R>(&self, what: &SurrealSelfRef<R>) -> SurrealUpdate<R>
    where
        R: SurrealProps + Clone,
        R::Remote: Clone,
    {
        SurrealUpdate(self.clone(), (*what).clone())
    }

    pub fn query(&self, query: impl IntoQuery) -> SurrealQuery<Client> {
        SurrealQuery(self.client.query(query))
    }

    pub fn create<R: Serialize + DeserializeOwned + Send + Sync, D: Serialize + Send + Sync>(
        &self,
        id: impl IntoResource<Vec<R>>,
        data: D,
    ) -> SurrealCreate<Client, D, R> {
        SurrealCreate(self.client.create(id).content(data))
    }
}

pub struct SurrealUpdate<R: SurrealProps>(SurrealToken, SurrealSelfRef<R>);

impl<R: 'static + SurrealProps + HasID> SurrealUpdate<R>
where
    <R as SurrealProps>::Remote: Clone + DeserializeOwned + Serialize + Send + Sync,
{
    /// Send the given data to the DB and update the local data if the new data still matches the original query
    ///
    /// Always uses MERGE because R may not include all fields of the underlying data
    pub fn with(self, data: R) -> Suspension {
        if let Ok(mut query) = TryInto::<SelectStatement>::try_into(self.1.state.get_selector()) {
            let id = (*data.id()).clone();

            query.what = Values(vec![Value::Thing(id.clone())]);

            logging::print_traffic(logging::Direction::Send, &data.get_remote());

            Suspension::from_future(async move {
                match self
                    .0
                    .client
                    .query("UPDATE $thing MERGE $data RETURN NONE")
                    .bind(("thing", id))
                    .bind(("data", data.get_remote()))
                    .query(query)
                    .await
                {
                    Ok(mut data) => match data.take::<Option<R::Remote>>(1) {
                        Ok(data) => {
                            logging::print_traffic(logging::Direction::Receive, &data);
                            self.1.set(data)
                        },
                        Err(error) => handle_error(error),
                    },
                    Err(error) => handle_error(error),
                };
            })
        } else {
            Suspension::new().0
        }
    }

    // /// Retrieve the current data from the DB and update this component if needed
    // pub fn refresh(self) -> Suspension {
    //     self.0.select((*self.1.id).clone()).store_to(&self.1.state)
    // }

    // /// Retrieve the current data from the DB and update this component or drop it if the record no longer exists
    // pub fn refresh_or_drop(self) -> Suspension {
    //     self.0
    //         .select((*self.1.id).clone())
    //         .store_or_drop(&self.1.state)
    // }
}

pub struct SurrealSelect<C: Connection, R: DeserializeOwned>(Select<'static, C, R>);

impl<Client, D> SurrealSelect<Client, Option<D>>
where
    Client: Connection,
    D: Clone + DeserializeOwned + Send + Sync + 'static,
{
    pub fn handle<F: 'static + FnOnce(Result<D>) -> ()>(self, f: F) -> Suspension {
        Suspension::from_future(async move { f(self.0.await) })
    }

    pub fn then<F: 'static + FnOnce(D) -> ()>(self, f: F) -> Suspension {
        Suspension::from_future(async move {
            self.0.await.ok().map(f);
        })
    }

    pub fn store_to(self, state: &UseStateHandle<Option<D>>) -> Suspension {
        let state = state.clone();
        Suspension::from_future(async move {
            match self.0.await {
                Ok(data) => state.set(Some(data)),
                Err(error) => handle_error(error),
            }
        })
    }
    
    // pub fn store_to_query_state(self, state: &QueryState<D>) -> Suspension {
    //     let state = state.clone();
    //     Suspension::from_future(async move {
    //         match self.0.await {
    //             Ok(data) => state.set(Ok(data)),
    //             Err(error) => handle_error(error),
    //         }
    //     })
    // }

    pub fn store_or_drop(self, state: &UseStateHandle<Option<D>>) -> Suspension {
        let state = state.clone();
        Suspension::from_future(async move {
            match self.0.await {
                Ok(data) => state.set(Some(data)),
                Err(_) => state.set(None),
            }
        })
    }

    pub fn append_to(self, result_list: QueryState<D>) -> Suspension {
        Suspension::from_future(async move {
            match self.0.await {
                Ok(data) => result_list.append(data),
                Err(error) => handle_error(error),
            }
        })
    }
}

pub struct SurrealQuery<C: Connection>(Query<'static, C>);

impl<C: Connection> SurrealQuery<C> {
    pub fn run(self) -> Suspension {
        Suspension::from_future(async move {
            let _ = self.0.await;
        })
    }

    pub fn query(mut self, query: impl IntoQuery) -> Self {
        self.0 = self.0.query(query);
        self
    }

    pub fn bind(mut self, bindings: impl Serialize) -> Self {
        self.0 = self.0.bind(bindings);
        self
    }

    pub fn store_to<R: 'static + DeserializeOwned>(self, state: UseStateHandle<Option<Vec<R>>>) -> Suspension {
        self.store_index(state, 0)
    }

    pub fn store_index<R: 'static + DeserializeOwned>(
        self,
        state: UseStateHandle<Option<Vec<R>>>,
        index: usize,
    ) -> Suspension {
        Suspension::from_future(async move {
            match self.0.await {
                Ok(mut response) => match response.take(index) {
                    Ok(data) => state.set(Some(data)),
                    Err(error) => handle_error(error),
                },
                Err(error) => handle_error(error),
            }
        })
    }

    pub fn store_multiple<R: 'static + DeserializeOwned>(
        self,
        states: Vec<(usize, UseStateHandle<Vec<R>>)>,
    ) -> Suspension {
        Suspension::from_future(async move {
            match self.0.await {
                Ok(mut response) => {
                    for (index, state) in states {
                        match response.take(index) {
                            Ok(data) => state.set(data),
                            Err(error) => handle_error(error),
                        }
                    }
                }
                Err(error) => handle_error(error),
            }
        })
    }

    pub fn store_response<R: 'static + DeserializeOwned>(self, state: UseStateHandle<Response>) -> Suspension {
        Suspension::from_future(async move {
            match self.0.await {
                Ok(response) => state.set(response),
                Err(error) => handle_error(error),
            }
        })
    }

    pub fn then<F: 'static + FnOnce(Response) -> ()>(self, f: F) -> Suspension {
        Suspension::from_future(async move {
            self.0.await.ok().map(f);
        })
    }
}

pub struct SurrealCreate<
    C: Connection,
    D: Serialize + Send + Sync,
    R: DeserializeOwned + Serialize + Send + Sync,
>(Content<'static, C, D, R>);

impl<C, D, R> SurrealCreate<C, D, R>
where
    C: Connection,
    D: 'static + Serialize + Send + Sync,
    R: 'static + Clone + DeserializeOwned + Serialize + Send + Sync,
{
    pub fn handle<F: 'static + FnOnce(Result<R>) -> ()>(self, f: F) -> Suspension {
        Suspension::from_future(async move { f(self.0.await) })
    }

    pub fn then<F: 'static + FnOnce(R) -> ()>(self, f: F) -> Suspension {
        Suspension::from_future(async move {
            self.0.await.ok().map(f);
        })
    }

    pub fn store_to(self, state: &UseStateHandle<Option<R>>) -> Suspension {
        let state = state.clone();
        Suspension::from_future(async move {
            match self.0.await {
                Ok(data) => state.set(Some(data)),
                Err(error) => handle_error(error),
            }
        })
    }

    pub fn store_or_drop(self, state: &UseStateHandle<Option<R>>) -> Suspension {
        let state = state.clone();
        Suspension::from_future(async move {
            match self.0.await {
                Ok(data) => state.set(Some(data)),
                Err(error) => handle_error(error),
            }
        })
    }

    pub fn append_to(self, result_list: QueryState<R>) -> Suspension {
        Suspension::from_future(async move {
            match self.0.await {
                Ok(data) => result_list.append(data),
                Err(error) => handle_error(error),
            }
        })
    }

    pub fn execute(self) -> Suspension {
        Suspension::from_future(async move {
            match self.0.await {
                Ok(_) => (),
                Err(error) => handle_error(error),
            };
        })
    }
}
