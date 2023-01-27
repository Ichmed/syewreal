use serde::{Deserialize, Serialize};
use syewreal::{
    components::QueryWithState,
    components::SurrealContext,
    hooks::{use_query_state, use_self_ref, use_surreal, use_surreal_login},
    props::id::ID,
    Client, Login, SurrealProps,
    props::{children::StaticChild, id::HasID}
};
use web_sys::HtmlInputElement;
use yew::prelude::*;

static CLIENT: Client = Client::init();

#[derive(SurrealProps, Properties, PartialEq, Clone)]
struct ToDoItemProps {
    #[id]
    id: Option<ID>,
    title: AttrValue,
    text: Option<AttrValue>,
    done: bool,
    img: Option<StaticChild<Img>>
}

#[derive(Deserialize, Serialize, Clone, PartialEq)]
struct Img {
    id: ID,
    url: String
}

impl HasID for Img {
    fn id(&self) -> ID {
        self.id.clone()
    }
}

#[function_component(ToDoItem)]
fn todo_item(props: &ToDoItemProps) -> Html {
    let sur = use_surreal();
    let self_ref = use_self_ref();
    
    let onclick = {
        let props = props.clone();
        use_callback(
            move |event: MouseEvent, ()| {
                sur.update(&self_ref).with(ToDoItemProps {
                    done: event
                        .target_dyn_into::<HtmlInputElement>()
                        .unwrap()
                        .checked(),
                    ..props.clone()
                });
            },
            (),
        )
    };

    html!(
        <div class="todo-item">
            <h3>{props.title.clone()}</h3>
            <div class="surreal-id">{props.id.clone()}</div>
            if let Some(text) = props.text.clone() {
                <div class="text">
                    {for text.split("\n\n").map(|x| html!(<p>{x}</p>))}
                </div>
            }
            if let Some(img) = props.img.clone() {
                <img src={img.url.clone()}/>
            }
            <div class="done-area"><input type="checkbox" checked={props.done.clone()} {onclick}/></div>
        </div>
    )
}

#[function_component(Home)]
fn home(_: &()) -> Html {
    let list_state = use_query_state::<ToDoItemProps>("SELECT * FROM item FETCH img");
    let sur = use_surreal();
    let onclick = {
        let state = list_state.clone();
        use_callback(
            move |_: MouseEvent, state| {
                sur.create(
                    "item".to_owned(),
                    ToDoItemPropsRemote {
                        done: false,
                        id: None,
                        text: None,
                        title: "Test".into(),
                        img: None
                    },
                )
                .append_to(state.clone());
            },
            state,
        )
    };

    let show_done = use_state(|| true);
    
    let filter = use_callback(|s: ToDoItemProps, show_done| {**show_done || !s.done}, show_done.clone());
    
    let show_done_handle = {
        let state = show_done.clone();
        use_callback(
            move |event: MouseEvent, state| {
                state.set(
                    event
                        .target_dyn_into::<HtmlInputElement>()
                        .unwrap()
                        .checked(),
                );
            },
            state,
        )
    };

    // let filter = match *show_done {
    //     true => None,
    //     false => Some(Callback::from(|d: ToDoItemPropsRemote| !d.done))
    // }.to_owned();

    html! {
        <>
        // {format!("{:?}", token.error.clone())}
            <div>
                <span>{"Show done:"}</span>
                <input onclick={show_done_handle} type="checkbox" checked={*show_done}/>
            </div>
            <div class="item-area">
                <QueryWithState<ToDoItem> filter={filter} state={list_state}/>
            </div>
            <button {onclick}>{"+"}</button>
        </>
    }
}

#[function_component]
fn App() -> Html {
    let credentials = Login {
        database: "todo",
        namespace: "todo",
        username: "Steve",
        password: "hunter2",
    };
    let token = use_surreal_login(&CLIENT, "localhost:8000".to_owned(), credentials);
    let fallback = html!(<span>{"Loading..."}</span>);
    html!{
        <SurrealContext {token} {fallback}>
           <Home/>
        </SurrealContext>
    }

}

fn main() {
    yew::Renderer::<App>::new().render();
}
