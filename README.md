# Usage

## Create a Connection
```rust
static CLIENT: Client = Client::init();
```
To connect to a database use the `use_surreal_login` hook. The hook returns a token that has to be used in a `<SurrealContext/>` context provider.
```rust
#[function_component]
fn MyComponent(props: &P) -> Html{
    let login = Database {
        username: "user",
        password: "user",
        database: "test",
        namespace: "test"
    };
    
    let token = use_surreal_login(&CLIENT, "localhost:8000".to_owned(), login);

    
    html! {
        <SurrealContext token={token}>
            // Your App goes here
        </SurrealContext>
    }
}
```
The `<SurrealContext/>` will suspend your app until the login was successful.

## Query Components
A `<Query/>` component will retrieve all database entries matching the given query and display them one after the other
```rust
html! {
    <SurrealContext token={token}>
        <Query<Inner> selector="select * from myTable"/>
    </SurrealContext>
}
```
`<Query<T>/>` has three parameters
- `selector`: Something that can be turned into a `SelectStatement`, this includes `String`s and surreal records (`Thing`s in the surreal source code)
- `parameters`: a Vec of `(String, String)` touples that will be bound to the query
- `filter`: a yew callback `Fn(T::Properties) -> bool` used for local filtering.

`<Query/>` stores the result of the query in an internally managed state. If you want to manipulat the state or use the same query result in multiple places you will have to use `<QueryWithState/>` in combination with the `use_query_state` hook
```rust
let list_state = use_query_state::<ToDoItemProps>("SELECT * FROM item");
html!{
    <QueryWithState<ToDoItem> state={list_state}/>
}
```
Because the state is externally managed the `<QueryWithState/>` component has no `selector` and `parameters` field.

### Properties
In order for the component `<Inner/>` to be rendered by `<Query/>` `Inner::Properties` needs to derive `SurrealProps` (in addition to `Properties`, `PartialEq` and `Clone`)
```rust
#[derive(SurrealProp, Properties, PartialEq, Clone)]
struct InnerProps {
    #[id] id: Option<ID>,
    name: AttrValue,
}


#[function_component]
fn Inner(props: &InnerProps) -> Html{
    // You know what to do here
}
```
The component's `id` property can be an `ID` or an `Option<ID>`, if you want to create new database entries from the components you should choose `Option<ID>` so you can set the id to `None`  on new entries.

### Local Properties
To add properties that can be set locally instead of retrieved from the server, mark them as `#[local]`
```rust
#[derive(SurrealProp, Properties, PartialEq, Clone)]
struct InnerProps {
    id: AttrValue,
    name: AttrValue,
    #[local]
    color: AttrValue
}
```
They will need to be set on the Wrapper Component and apply to all `Inner` components
```rust
<SurrealList<Inner> selector="select * from myTable" color="red"/>
```

### ForeignKeys
There are two ways to deal with foreign keys:
A) Use a property of type `ForeignKey` (this is just an alias of `ID` but more readable)

B) Use a property of type `StaticChild<T>` where `T` is a deserializable struct. This allows for the data to be retrieved in one go with the `FETCH` keyword but will only write the id of the fetched data to surreal when updated/created


## "Raw" Database access
Instead of rendering Components directly with a `<Query/>` you can use the `use_surreal()` hook to:
- `select`: fetch arbitrary data from surreal using a `Selector`
- `update`: update a components underlying data (local and remote). This always uses the `MERGE` mode since there is no guarantee that the local struct contains all fields of the remote table.
- `create`: create a new database entry and use convinience methods to store the new entry locally
- `query`: Execute an arbitrary SQL query and store the results

```rust
use_surreal().create(
    "item".to_owned(),
    ToDoItemPropsRemote {
        done: false,
        // Note how having the type of id be Option<ID> 
        // enables you to delegate id creation to the DB
        id: None,
        text: None,
        title: "Test".into(),
        img: None
    },
).execute();
```

### Self Refs
The `use_surreal().update()` method takes a `SurrealSelfRef` as its argument, this can be obtained from inside a component by using the hook `use_self_ref()`.

When the update is executed the data returned from the database will be used to replace the properties of the component iff the new data still matches the original selector, otherwise the data is dropped from local storage.

**Note:** It is highly recomended to use the `use_update_callback` hook when creating self-updating components.
