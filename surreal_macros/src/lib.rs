use proc_macro::TokenStream;
use quote::quote;
use surrealdb::sql::{parse, Statement};
use syn::{parse_macro_input, Data, DeriveInput, Field, Ident, __private::Span};

/// Parse the sql during compile time to safely unwrap it during runtime
#[proc_macro]
pub fn sqlx(item: TokenStream) -> TokenStream {
    match parse(item.to_string().as_str()) {
        Err(err) => panic!("Syntax error: {}", err.to_string().as_str()),
        Ok(_) => {
            let item: quote::__private::TokenStream = item.into();

            quote!{
                {
                    let query: surrealdb::sql::Query = surrealdb::sql::parse(stringify!(#item)).expect("Checked during compile time");
                    query
                }
            }.into()
        }
    }
}

#[proc_macro]
#[allow(non_snake_case)]
pub fn SELECT(item: TokenStream) -> TokenStream {
    let raw = "SELECT ".to_owned() + &item.to_string();
    match parse(raw.as_str()) {
        Err(err) => panic!("Syntax error: {}", err.to_string().as_str()),
        Ok(stmt) => match stmt.to_vec().first() {
            Some(Statement::Select(_)) => {
                quote!{
                    match surrealdb::sql::parse(#raw)
                        .expect("Checked during compile time").first() {
                            Some(Statement::Select(x)) => x.clone(),
                            _ => panic!()
                        }
                }.into()
            },
            Some(_) => panic!("{} is not a SELECT statement", item.to_string()),
            None => panic!("Empty statement")
        }
    }
}

#[proc_macro_derive(SurrealProps, attributes(local, fallback, id))]
pub fn derive_surreal_props(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let Data::Struct(data) = input.data {
        let name = input.ident;

        let fields: Vec<_> = data.fields.into_iter().collect();

        let (local_data, rest) = extract_attr(fields, "local");


        let remote_name = create_ident(&name, "Remote");
        let local_name = create_ident(&name, "Local");
        let local_with_state_name = create_ident(&name, "LocalWithState");
        
        // Get the 'id' field used for keying lists
        let (id, rest) = find_optional_field(rest, "id");

        let id_getter = id.clone().map(|field| {
            let id_ident = field.ident;

            quote! {
                impl syewreal::props::id::HasID for #name {
                    fn id(&self) -> syewreal::props::id::ID {
                        self.#id_ident.as_ref().unwrap().clone()
                    }
                }
                
                impl syewreal::props::id::HasID for #remote_name {
                    fn id(&self) -> syewreal::props::id::ID {
                        self.#id_ident.as_ref().unwrap().clone()
                    }
                }
            }
        });

        let (fallback, rest) = extract_optional_field(rest, "fallback");
        let fallback_ident = fallback.as_ref().map(|field| field.ident.clone());

        let fallback_getter = fallback_ident.clone().map(|field| {

            quote! {
                fn get_fallback(&self) -> Option<yew::Html> {
                    Some(self.#field.clone())
                }
            }
        });

        let fallback_construct_assignment = fallback_ident.clone().map(|field| {
            quote!{
                #field: local.#field.clone(),
            }
        });
        
        let fallback_state_assignment = fallback_ident.clone().map(|field| {
            quote!{
                #field: self.#field.clone(),
            }
        });

        let (attrs, rest) = extract_by_type(rest, "AttrValue");
        let attr_idents = attrs
            .iter()
            .map(|x| x.ident.clone().unwrap())
            .collect::<Vec<_>>();

        let (opt_attrs, rest) = extract_by_type(rest, "Option<AttrValue>");
        let opt_attr_idents = opt_attrs
            .iter()
            .map(|x| x.ident.clone().unwrap())
            .collect::<Vec<_>>();

        let remote_data = rest;


        let local_idents = get_idents(&local_data);
        let remote_idents = get_idents(&remote_data);

        let expanded = quote! {


            #[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq)]
            struct #remote_name {
                #(#remote_data,)*
                #(#attr_idents: String,)*
                #(#opt_attr_idents: Option<String>,)*
            }

            #[derive(Clone, yew::Properties, PartialEq)]
            struct #local_name {
                #(#local_data,)*
                selector: syewreal::props::selector::Selector,
                #[prop_or_default]
                parameters: syewreal::props::selector::Parameters,
                #[prop_or_default]
                filter: Option<yew::Callback<#name, bool>>,
                #fallback
            }

            #[derive(Clone, yew::Properties, PartialEq)]
            struct #local_with_state_name {
                #(#local_data,)*
                #[prop_or_default]
                parameters: syewreal::props::selector::Parameters,
                #[prop_or_default]
                filter: Option<yew::Callback<#name, bool>>,
                state: syewreal::hooks::QueryState<#remote_name>,
                #fallback
            }


            impl syewreal::SurrealProps for #name {
                type Remote = #remote_name;
                type Local = #local_name;
                type LocalWithState = #local_with_state_name;

                fn construct(
                    remote: Self::Remote,
                    local: Self::LocalWithState
                ) -> Self {
                    Self {
                        #(#remote_idents : remote.#remote_idents.clone(),)*

                        // Convert Strings into AttrValues
                        #(#attr_idents: remote.#attr_idents.into(),)*
                        #(#opt_attr_idents: remote.#opt_attr_idents.map(AttrValue::from),)*

                        #(#local_idents : local.#local_idents,)*
                        #fallback_construct_assignment
                    }
                }

                fn get_remote(&self) -> Self::Remote {
                    Self::Remote {
                        #(#remote_idents : self.#remote_idents.clone(),)*
                        #(#attr_idents: self.#attr_idents.as_str().to_owned(),)*
                        #(#opt_attr_idents: self.#opt_attr_idents.as_ref().map(|x| x.as_str().to_owned()),)*
                    }
                }

            }

            #id_getter
            
            impl syewreal::props::surreal_props::PropsNoState<#name, #remote_name, #local_with_state_name> for #local_name {
                fn with_state(&self, state: syewreal::hooks::QueryState<#remote_name>) -> #local_with_state_name {
                    #local_with_state_name {
                        #(#local_idents : self.#local_idents.clone(),)*
                        parameters: self.parameters.clone(),
                        filter: self.filter.clone(),
                        #fallback_state_assignment
                        state: state.clone()
                    }
                }

                fn get_selector(&self) -> syewreal::props::selector::Selector {
                    self.selector.clone()
                }
            }
            
            impl syewreal::props::surreal_props::PropsWithState<#name, #remote_name> for #local_with_state_name {
                fn get_state(&self) -> syewreal::hooks::QueryState<#remote_name> {
                    self.state.clone()
                }

                fn get_parameters(&self) -> syewreal::props::selector::Parameters {
                    self.parameters.clone()
                }

                fn get_filter(&self) -> Option<yew::Callback<#name, bool>> {
                    self.filter.clone()
                }

                #fallback_getter
            }
        };

        // Hand the output tokens back to the compiler.
        proc_macro::TokenStream::from(expanded)
    } else {
        proc_macro::TokenStream::default()
    }
}

fn create_ident(ident: &Ident, suffix: &str) -> Ident {
    Ident::new(&(ident.to_string() + &suffix.to_owned()), Span::call_site()).into()
}

fn get_idents(fields: &Vec<Field>) -> Vec<Ident> {
    fields
        .iter()
        .map(|x| x.ident.as_ref().unwrap().clone())
        .collect::<Vec<_>>()
}

fn extract_by_type(fields: Vec<Field>, field_type: &str) -> (Vec<Field>, Vec<Field>) {
    fields
        .into_iter()
        .partition(|x| x.ty == syn::parse_str(field_type).expect("is valid type"))
}

fn find_optional_field(fields: Vec<Field>, attr_name: &str) -> (Option<Field>, Vec<Field>) {
    let (req, rest) = extract_attr(fields, attr_name);

    if req.len() > 1 {
        panic!("Only one field may be marked as '{}'.", attr_name)
    }

    (req.first().cloned(), [req, rest].concat())
}

fn extract_optional_field(fields: Vec<Field>, attr_name: &str) -> (Option<Field>, Vec<Field>) {
    let (req, rest) = extract_attr(fields, attr_name);

    if req.len() > 1 {
        panic!("Only one field may be marked as '{}'.", attr_name)
    }

    (req.first().cloned(), rest)
}

/// split off all fields with a certain attr and strip that attr out of them
fn extract_attr(fields: Vec<Field>, attr_name: &str) -> (Vec<Field>, Vec<Field>) {
    let (mut head, rest): (Vec<_>, Vec<_>) = fields
        .into_iter()
        .partition(|x| x.attrs.iter().any(|attr| attr.path.is_ident(attr_name)));

    head = strip_attr(head, attr_name);

    (head, rest)
}

fn strip_attr(mut fields: Vec<Field>, attr_name: &str) -> Vec<Field> {
    fields
        .iter_mut()
        .for_each(|field| field.attrs.retain(|attr| !attr.path.is_ident(attr_name)));
    fields
}
