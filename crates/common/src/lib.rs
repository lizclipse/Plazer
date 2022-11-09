pub mod api;
mod graph;

use std::ops::{Deref, DerefMut};

pub use graph::*;

/// A container type that allows for a value to be bundled with its underlying data.
///
/// Primarily, this is used to do zero-copy deserialisation
/// while still allowing for owned-semantics.
#[derive(Debug)]
pub struct Container<D, V> {
    _data: D,
    value: V,
}

impl<D, V> Container<D, V> {
    pub fn new(data: D, value: V) -> Self {
        Self { _data: data, value }
    }
}

impl<D, V> From<(D, V)> for Container<D, V> {
    fn from((data, value): (D, V)) -> Self {
        Container::new(data, value)
    }
}

impl<D, T> Deref for Container<D, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<D, T> DerefMut for Container<D, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
