use std::collections::HashMap;
use std::collections::HashSet;

pub struct Node {
    pub(crate) id: String,
    pub(crate) account_id: String,
    
    pub(crate) text_short: Option<String>,
    pub(crate) text_long: Option<String>,
    pub(crate) attachments: Vec<Attachment>,
    pub(crate) label: Option<String>,
    pub(crate) tags: HashSet<String>,

    pub(crate) edge_in: HashMap<Edge, u64>,
    pub(crate) edge_out: HashMap<Edge, u64>,
}

impl Node {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn account_id(&self) -> &str {
        &self.account_id
    }

    pub fn attachments(&self) -> &[Attachment] {
        &self.attachments
    }

    pub fn tags(&self) -> &HashSet<String> {
        &self.tags
    }

    pub fn count_edge_in(&self, edge: &Edge) -> u64 {
        *self.edge_in.get(edge).unwrap_or(&0)
    }

    pub fn count_edge_out(&self, edge: &Edge) -> u64 {
        *self.edge_out.get(edge).unwrap_or(&0)
    }
}

#[derive(Eq, Hash, PartialEq)]
pub enum Kind {
    Post,
    Board,
    Collection,
}

pub enum Attachment {
    Image,
    Audio,
    LiveAudio,
    Video,
    LiveVideo,
    File,
}

#[derive(Eq, Hash, PartialEq)]
pub enum Edge {}
