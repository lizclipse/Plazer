use std::{collections::VecDeque, fmt::Debug, future::Future, mem};

use async_graphql::{
    connection::{Connection, CursorType},
    OutputType,
};

use super::OpaqueCursor;
use crate::prelude::*;

pub fn encoded_cursor(id: impl Into<String>) -> String {
    OpaqueCursor(id.into()).encode_cursor()
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PaginatorState<C> {
    Initial,
    Next(C),
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paginator<F, C> {
    state: PaginatorState<C>,
    next_page: F,
    reversed: bool,
}

impl<T, F, C, Fut> Paginator<F, C>
where
    T: OutputType + Send + Sync + Unpin + Clone + Debug,
    F: Fn(Option<C>) -> Fut,
    C: CursorType + Send + Sync + Unpin,
    Fut: Future<Output = Result<Connection<C, T>>>,
{
    pub fn new(next_page: F) -> Self {
        Self {
            state: PaginatorState::Initial,
            next_page,
            reversed: false,
        }
    }

    #[must_use]
    pub fn reversed(mut self) -> Self {
        self.reversed = true;
        self
    }

    pub async fn next(&mut self) -> Option<Result<Vec<T>>> {
        let mut state = PaginatorState::Done;
        mem::swap(&mut state, &mut self.state);
        let res = match state {
            PaginatorState::Initial => (self.next_page)(None).await,
            PaginatorState::Next(cursor) => (self.next_page)(Some(cursor)).await,
            PaginatorState::Done => return None,
        };

        match res {
            Ok(connection) => {
                let (res, state) = if self.reversed {
                    extract_backward_page(connection)
                } else {
                    extract_forward_page(connection)
                };
                self.state = state;
                res
            }
            Err(e) => {
                self.state = PaginatorState::Done;
                Some(Err(e))
            }
        }
    }
}

type PaginationExtractResult<T, C> = (Option<Result<Vec<T>>>, PaginatorState<C>);

fn extract_forward_page<T, C>(
    Connection {
        mut edges,
        has_next_page,
        ..
    }: Connection<C, T>,
) -> PaginationExtractResult<T, C>
where
    T: OutputType + Send + Sync + Clone + Debug,
    C: CursorType + Send + Sync,
{
    println!(
        "{edges:#?} {has_next_page:?}",
        edges = edges
            .iter()
            .map(|edge| edge.node.clone())
            .collect::<Vec<_>>()
    );
    let Some(last) = edges.pop() else {
        return (None, PaginatorState::Done);
    };
    let state = if has_next_page {
        PaginatorState::Next(last.cursor)
    } else {
        PaginatorState::Done
    };
    let mut items: Vec<_> = edges.into_iter().map(|edge| edge.node).collect();
    items.push(last.node);
    (Some(Ok(items)), state)
}

fn extract_backward_page<T, C>(
    Connection {
        edges,
        has_previous_page,
        ..
    }: Connection<C, T>,
) -> PaginationExtractResult<T, C>
where
    T: OutputType + Send + Sync,
    C: CursorType + Send + Sync,
{
    let mut edges: VecDeque<_> = edges.into();
    let Some(first) = edges.pop_front() else {
        return (None, PaginatorState::Done);
    };
    let state = if has_previous_page {
        PaginatorState::Next(first.cursor)
    } else {
        PaginatorState::Done
    };
    let mut items = vec![first.node];
    items.extend(edges.into_iter().map(|edge| edge.node));
    (Some(Ok(items)), state)
}
