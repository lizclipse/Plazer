pub mod api;
mod graph;

use std::ops::Deref;

pub use graph::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    id: String,
    name: Option<String>,
}

impl Account {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|x| &**x)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Post(Node);

impl Post {
    pub fn text_short(&self) -> Option<&String> {
        self.text_short.as_ref()
    }

    pub fn text_long(&self) -> Option<&String> {
        self.text_long.as_ref()
    }

    pub fn category(&self) -> Option<&String> {
        self.label.as_ref()
    }
}

impl From<Node> for Post {
    fn from(node: Node) -> Self {
        Self(node)
    }
}

impl Deref for Post {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Board(Node);

impl From<Node> for Board {
    fn from(node: Node) -> Self {
        Self(node)
    }
}

impl Deref for Board {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Collection(Node);

impl From<Node> for Collection {
    fn from(node: Node) -> Self {
        Self(node)
    }
}

impl Deref for Collection {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
