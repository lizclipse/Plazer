use std::collections::HashMap;
use std::collections::HashSet;

use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;

#[derive(Debug, Serialize, Deserialize)]
#[skip_serializing_none]
pub struct Node {
    pub(crate) id: String,
    pub(crate) account_id: String,

    pub(crate) text_short: Option<String>,
    pub(crate) text_long: Option<String>,
    pub(crate) attachments: Vec<Attachment>,
    pub(crate) label: Option<String>, // post category
    pub(crate) tags: Option<HashSet<String>>,

    pub(crate) edge_in: Option<HashMap<Edge, u64>>,
    pub(crate) edge_out: Option<HashMap<Edge, u64>>,
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

    pub fn has_tag(&self, tag: impl AsRef<str>) -> bool {
        self.tags
            .as_ref()
            .map(|t| t.contains(tag.as_ref()))
            .unwrap_or(false)
    }

    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tags = self.tags.get_or_insert_with(Default::default);
        tags.insert(tag.into());
    }

    pub fn remove_tag(&mut self, tag: impl AsRef<str>) {
        if let Some(tags) = &mut self.tags {
            tags.remove(tag.as_ref());

            // Clean up if empty.
            if tags.is_empty() {
                self.tags.take();
            }
        }
    }

    pub fn count_edges_in(&self, edge: &Edge) -> u64 {
        Self::count_edges(&self.edge_in, edge)
    }

    pub fn add_edge_in(&mut self, edge: impl Into<Edge>) {
        Self::add_edge(&mut self.edge_in, edge);
    }

    pub fn remove_edge_in(&mut self, edge: &Edge) {
        Self::remove_edge(&mut self.edge_in, edge);
    }

    pub fn count_edge_out(&self, edge: &Edge) -> u64 {
        Self::count_edges(&self.edge_out, edge)
    }

    pub fn add_edge_out(&mut self, edge: impl Into<Edge>) {
        Self::add_edge(&mut self.edge_out, edge);
    }

    pub fn remove_edge_out(&mut self, edge: &Edge) {
        Self::remove_edge(&mut self.edge_out, edge);
    }

    fn count_edges(edges: &Option<HashMap<Edge, u64>>, edge: &Edge) -> u64 {
        *edges.as_ref().and_then(|e| e.get(edge)).unwrap_or(&0)
    }

    fn add_edge(edges: &mut Option<HashMap<Edge, u64>>, edge: impl Into<Edge>) {
        let edges = edges.get_or_insert_with(Default::default);
        edges
            .entry(edge.into())
            .and_modify(|c| *c = *c + 1)
            .or_insert(1);
    }

    fn remove_edge(opt_edges: &mut Option<HashMap<Edge, u64>>, edge: &Edge) {
        let edges = match opt_edges {
            Some(edges) => edges,
            None => return,
        };

        let entry = match edges.get_mut(edge) {
            Some(entry) => entry,
            None => return,
        };

        // UNDERFLOW: We always start at 1, and remove any entries that hit 0,
        // so no entry will ever be 0 at this point.
        *entry = *entry - 1;

        if *entry == 0 {
            edges.remove(edge);
        }

        if edges.is_empty() {
            opt_edges.take();
        }
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum Kind {
    Post,
    Board,
    Collection,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Attachment {
    Image,
    Audio,
    LiveAudio,
    Video,
    LiveVideo,
    File,
}

#[derive(Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum Edge {}
