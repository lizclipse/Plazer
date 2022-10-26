mod graph;

use std::ops::Deref;

pub use graph::*;

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

pub struct Post(Node);

impl Post {
    pub fn text_short(&self) -> Option<&str> {
        self.text_short.as_ref().map(|x| &**x)
    }

    pub fn text_long(&self) -> Option<&str> {
        self.text_long.as_ref().map(|x| &**x)
    }

    pub fn category(&self) -> Option<&str> {
        self.label.as_ref().map(|x| &**x)
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
