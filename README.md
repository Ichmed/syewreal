# Usage

### Create a Connection
```rust
// This is the normal surreal client, will probably be reexported in the future
static CLIENT: Surreal<Client> = Surreal::init();
```
To connect to a database use the `use_surreal_login` hook. The hook return a token that has to be used in a `SurrealContext` context provider.
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
            <SurrealList<Inner> selector="select * from myTable"/>
        </SurrealContext>
    }
}
```
### Display Components
A `SurrealList` component will retrieve all database entries matching the given query and display them one after the other (the component is called list but the rendered DOM elements can be anything not just list elements)

A `SurrealComponent` will always only display the first result of the query (The `LIMIT` of the query is automatically set to `1`) 
```rust
html! {
    <SurrealContext token={token}>
        <SurrealList<Inner> selector="select * from myTable"/>
    </SurrealContext>
}
```
The `Inner` component's props need to derive `SurrealProp`
```rust

#[derive(SurrealProp, Properties, PartialEq, Clone)]
struct InnerProps {
    id: AttrValue,
    name: AttrValue,
}


#[function_component]
fn Inner(props: &InnerProps) -> Html{
    // You know what to do here
}
```
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
