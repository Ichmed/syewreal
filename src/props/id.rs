use std::{fmt::Display, ops::Deref};

use serde::{de::Error, de::Visitor, Deserialize, Deserializer, Serialize};
use surrealdb::sql::Thing;

pub trait HasID {
    fn id(&self) -> ID;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ID(Thing);
pub type ForeignKey = ID;

impl Deref for ID {
    type Target = Thing;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for ID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for ID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ID {
    fn deserialize<D>(deserializer: D) -> Result<ID, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(ThingVisitor)
    }
}

pub struct ThingVisitor;

static MAP_FIELDS: &[&str] = &[&"tb", &"id"];

impl<'de> Visitor<'de> for ThingVisitor {
    type Value = ID;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("\"table:id\" or {\"tb\": id, \"id\": id}")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let split: Vec<_> = v.split(":").collect();
        if split.len() != 2 {
            Err(E::invalid_length(split.len(), &self))
        } else {
            Ok(ID((split[0], split[1]).into()))
        }
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut tb: Option<&str> = None;
        let mut id = None;
        while let Some((key, value)) = map.next_entry()? {
            match key {
                "tb" => tb = Some(value),
                "id" => id = Some(value),
                _ => return Err(A::Error::unknown_field(key, MAP_FIELDS)),
            };
        }
        match (tb, id) {
            (Some(tb), Some(id)) => Ok(ID((tb, id).into())),
            (None, _) => Err(A::Error::missing_field("tb")),
            (Some(_), None) => Err(A::Error::missing_field("id")),
        }
    }
}
