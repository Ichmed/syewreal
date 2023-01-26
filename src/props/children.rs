use std::ops::Deref;

use serde::{Deserialize, Serialize};

use super::id::{ForeignKey, HasID};

/// A child that was "fetched"
#[derive(Clone, PartialEq)]
pub struct StaticChild<T: Clone + PartialEq>(ForeignKey, T);

impl<T: Clone + PartialEq> Serialize for StaticChild<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, T: Clone + PartialEq + Deserialize<'de> + HasID> Deserialize<'de> for StaticChild<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        let inner = T::deserialize(deserializer)?;
        Ok(Self(inner.id(), inner))
    }
}


impl<T: Clone + PartialEq> Deref for StaticChild<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.1
    }
}
