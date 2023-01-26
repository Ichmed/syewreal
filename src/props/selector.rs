use surrealdb::{sql::{statements::SelectStatement, Statement, Query, Thing, Values}, opt::IntoQuery};
use yew::{html::IntoPropValue, AttrValue};


#[derive(Clone, PartialEq, Debug)]
pub struct Selector {
    pub base: Option<SelectStatement>,
}

impl TryFrom<Selector> for SelectStatement {
    type Error = ();
    fn try_from(value: Selector) -> Result<Self, Self::Error> {
        value.base.ok_or(())
    }
}

impl From<Vec<Statement>> for Selector {
    fn from(value: Vec<Statement>) -> Self {
        Self {
            base: match value.first() {
                Some(Statement::Select(x)) => Some(x.clone()),
                _ => None,
            },
        }
    }
}

impl IntoQuery for Selector {
    fn into_query(self) -> surrealdb::Result<Vec<surrealdb::sql::Statement>> {
        match self.base {
            Some(x) => x.into_query(),
            None => Ok(vec![]),
        }
    }
}

impl IntoPropValue<Selector> for SelectStatement {
    fn into_prop_value(self) -> Selector {
        vec![Statement::Select(self)].into()
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

impl IntoPropValue<Selector> for Thing {
    fn into_prop_value(self) -> Selector {
        Selector{
            base: Some(SelectStatement{
                what: Values(vec![self.into()]),
                ..Default::default()
            })
        }        
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