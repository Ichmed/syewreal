use proc_macro::TokenStream;
use quote::quote;
use surrealdb::sql::parse;
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

#[proc_macro_derive(SurrealProp, attributes(local, selector, id))]
pub fn derive_surreal_prop(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    if let Data::Struct(data) = input.data {
        let name = input.ident;

        let fields: Vec<_> = data.fields.into_iter().collect();

        let (local_data, rest) = extract_attr(fields, "local");

        let (selector_field_pub, rest) = extract_required_field(rest, "selector");
        let selector_ident_pub = selector_field_pub.as_ref().map(|x| x.ident.clone());

        let selector_field_priv = match selector_field_pub {
            Some(t) => quote!(#t),
            None => quote!(selector: syewreal::surreal::Selector),
        };

        let selector_construction = match &selector_ident_pub {
            Some(Some(t)) => {
                quote!(yew::html::IntoPropValue::<syewreal::surreal::Selector>::into_prop_value(self.#t.clone()))
            }
            _ => quote!(self.selector.clone()),
        };

        let selector_assignment = &selector_ident_pub.map(|x| {
            let x = x.unwrap();
            quote!(#x: local.#x,)
        });

        // Get the 'id' field used for keying lists
        let (id, rest) = find_optional_field(rest, "id");

        let id_getter = if let Some(id_field) = id.clone() {
            let id_ident = id_field.ident;

            quote! {
                Some(self.#id_ident.clone())
            }
        } else {
            quote!(None)
        };

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

        let remote_name = create_ident(&name, "Remote");
        let local_name = create_ident(&name, "Local");

        let local_idents = get_idents(&local_data);
        let remote_idents = get_idents(&remote_data);

        let expanded = quote! {

            #[derive(serde::Deserialize, serde::Serialize, Clone)]
            struct #remote_name {
                #(#remote_data,)*
                #(#attr_idents: String,)*
                #(#opt_attr_idents: Option<String>,)*
            }

            #[derive(Clone, yew::Properties, PartialEq)]
            struct #local_name {
                #(#local_data,)*
                // #token_field_priv,
                #selector_field_priv,
                #[prop_or_default]
                parameters: syewreal::surreal::Parameters,
            }

            impl syewreal::surreal::SurrealProp for #name {
                type Remote = #remote_name;
                type Local = #local_name;

                fn construct(
                    remote: Self::Remote,
                    local: Self::Local
                ) -> Self {
                    Self {
                        #(#remote_idents : remote.#remote_idents,)*

                        // Convert Strings into AttrValues
                        #(#attr_idents: remote.#attr_idents.into(),)*
                        #(#opt_attr_idents: remote.#opt_attr_idents.map(AttrValue::from),)*

                        #(#local_idents : local.#local_idents,)*

                        // #token_assignment

                        #selector_assignment
                    }
                }

                fn get_id(&self) -> Option<AttrValue> {
                    #id_getter
                }
            }

            impl syewreal::surreal::SurrealLocalProp for #local_name {

                // fn get_token(&self) -> &SurrealToken {
                //     #token_construction
                // }

                fn get_selector(&self) -> syewreal::surreal::Selector {
                    #selector_construction
                }

                fn get_parameters(&self) -> syewreal::surreal::Parameters {
                    self.parameters.clone()
                }
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

fn extract_required_field(fields: Vec<Field>, attr_name: &str) -> (Option<Field>, Vec<Field>) {
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
