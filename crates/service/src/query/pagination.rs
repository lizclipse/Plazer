#[cfg(test)]
pub mod testing;
#[cfg(test)]
mod tests;

use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use async_graphql::{connection::CursorType, ID};
use serde::{de::DeserializeOwned, Serialize};

use crate::prelude::*;

pub const MAX_LIMIT: i64 = 100;
pub const PAGE_EXTRA: i64 = 2;
pub const SRQL_ORDER_ASC: bool = true;
pub const SRQL_ORDER_DESC: bool = false;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PaginationArgs {
    pub after: Option<String>,
    pub before: Option<String>,
    pub first: Option<i32>,
    pub last: Option<i32>,
}

impl PaginationArgs {
    pub fn validate<Cursor>(self) -> Result<PaginationInput<Cursor>>
    where
        Cursor: CursorType + Debug + Default + Clone,
    {
        self.try_into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PaginationDirection {
    First(i64),
    Last(i64),
}

impl PaginationDirection {
    pub fn limit(&self) -> i64 {
        match self {
            Self::First(lim) | Self::Last(lim) => *lim,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct PaginationInput<Cursor>
where
    Cursor: CursorType + Debug + Default + Clone,
{
    direction: Option<PaginationDirection>,
    after: Option<Cursor>,
    before: Option<Cursor>,
}

#[cfg(test)]
impl<Cursor> PaginationInput<Cursor>
where
    Cursor: CursorType + Debug + Default + Clone,
{
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn forward(mut self, first: i64) -> Self {
        self.direction = Some(PaginationDirection::First(first));
        self
    }

    #[must_use]
    pub fn backward(mut self, last: i64) -> Self {
        self.direction = Some(PaginationDirection::Last(last));
        self
    }

    #[must_use]
    pub fn set_after(mut self, after: Option<Cursor>) -> Self {
        self.after = after;
        self
    }

    #[must_use]
    pub fn set_before(mut self, before: Option<Cursor>) -> Self {
        self.before = before;
        self
    }
}

impl<Cursor> TryFrom<PaginationArgs> for PaginationInput<Cursor>
where
    Cursor: CursorType + Debug + Default + Clone,
{
    type Error = Error;

    fn try_from(
        PaginationArgs {
            after,
            before,
            first,
            last,
        }: PaginationArgs,
    ) -> Result<Self> {
        fn parse_cursor<Cursor>(cursor: Option<String>) -> Result<Option<Cursor>>
        where
            Cursor: CursorType + Debug + Default + Clone,
        {
            cursor
                .map(|cursor| {
                    Cursor::decode_cursor(&cursor)
                        .map_err(|e| Error::PaginationInvalid(e.to_string()))
                })
                .transpose()
        }

        // This is essentially a re-impl of the logic provided by async-graphql's
        // Connection validation, but it uses the direction enum to make the
        // logic a bit clearer down the line.
        let direction = match (first, last) {
            (Some(_), Some(_)) => {
                return Err(Error::PaginationInvalid(
                    "The \"first\" and \"last\" parameters cannot exist at the same time".into(),
                ))
            }
            (Some(first), None) if first >= 0 => Some(PaginationDirection::First(i64::from(first))),
            (Some(_), None) => {
                return Err(Error::PaginationInvalid(
                    "The \"first\" parameter must be a non-negative number".into(),
                ))
            }
            (None, Some(last)) if last >= 0 => Some(PaginationDirection::Last(i64::from(last))),
            (None, Some(_)) => {
                return Err(Error::PaginationInvalid(
                    "The \"last\" parameter must be a non-negative number".into(),
                ))
            }
            (None, None) => None,
        };

        Ok(PaginationInput {
            direction,
            after: parse_cursor(after)?,
            before: parse_cursor(before)?,
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct ResultSliceOptions {
    reverse_results: bool,
    limit: i64,
    after: Option<ID>,
    before: Option<ID>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct PaginationOptions {
    pub cond: Option<srql::Cond>,
    pub order: Option<srql::Order>,
    pub limit: Option<srql::Limit>,
    pub result_slice_opts: ResultSliceOptions,
}

impl From<(PaginationInput<OpaqueCursor<String>>, &str)> for PaginationOptions {
    fn from(
        (
            PaginationInput {
                direction,
                after,
                before,
            },
            table_name,
        ): (PaginationInput<OpaqueCursor<String>>, &str),
    ) -> Self {
        // First we pull out the cursors into their own expressions and the native
        // ID type to make it easier to work with.
        let (after_expr, after) = after
            .map(|OpaqueCursor(after)| {
                (
                    srql::Expression::Binary {
                        l: srql::field("id").into(),
                        o: srql::Operator::LessThanOrEqual,
                        r: srql::Thing::from((table_name, after.as_str())).into(),
                    },
                    ID(after),
                )
            })
            .unzip();

        let (before_expr, before) = before
            .map(|OpaqueCursor(before)| {
                (
                    srql::Expression::Binary {
                        l: srql::field("id").into(),
                        o: srql::Operator::MoreThanOrEqual,
                        r: srql::Thing::from((table_name, before.as_str())).into(),
                    },
                    ID(before),
                )
            })
            .unzip();

        // We always have a limit when making search queries, but this will
        // silently clamp the limit to the max if the caller has naughtily
        // requested more than the max. If this confuses anyone, they can just
        // read the docs I guess...
        let limit = std::cmp::min(
            direction
                .as_ref()
                .map_or(MAX_LIMIT, PaginationDirection::limit),
            MAX_LIMIT,
        );

        // Working out the internals of SurrealDB's syntax tree was hell, but
        // I refuse to use string interpolation for this.
        PaginationOptions {
            cond: match (after_expr, before_expr) {
                (Some(after), Some(before)) => srql::Cond(
                    srql::Expression::Binary {
                        l: after.into(),
                        o: srql::Operator::And,
                        r: before.into(),
                    }
                    .into(),
                )
                .into(),
                (Some(after), None) => srql::Cond(after.into()).into(),
                (None, Some(before)) => srql::Cond(before.into()).into(),
                (None, None) => None,
            },
            // GraphQL's pagination spec requires that sorting is the same
            // for both directions, but figuring out how to make that work in
            // a query would require an understanding beyond mere mortals.
            // Instead, we just fix it later before returning the results.
            order: Some(srql::Order {
                order: srql::field("id"),
                direction: match direction {
                    Some(PaginationDirection::Last(_)) => SRQL_ORDER_ASC,
                    _ => SRQL_ORDER_DESC,
                },
                ..Default::default()
            }),
            // This seems a bit odd at first, but this is how we can tell if there's
            // previous and next pages. The idea is something like this:
            //
            // |   0   | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |   9    |
            // | after |             page              | before |
            //
            // The cursors are inclusive, and if we get those items back, we know
            // that there's content before and after the page. When a cursor is
            // not present, then there cannot be content before (for an after cursor)
            // or after (for a before cursor), but this overfetching will still result
            // in some extra items being returned in the results iff there is a
            // previous or next page.
            //
            // I'm sure there's a better way to do this, but this the most robust way
            // to do it I can think of, especially if we want to do something like search
            // by date. Since the ID is a UUIDv7, which includes a timestamp in the
            // upper bits, we can generate cursors for a given date range and then
            // use those to paginate through the results. Time has always felt like
            // one of the most important things to organise by to me, so this is a
            // pretty important feature.
            limit: Some(srql::Limit(srql::Number::Int(limit + PAGE_EXTRA).into())),
            result_slice_opts: ResultSliceOptions {
                reverse_results: matches!(direction, Some(PaginationDirection::Last(_))),
                limit,
                after,
                before,
            },
        }
    }
}

impl From<(Option<PaginationInput<OpaqueCursor<String>>>, &str)> for PaginationOptions {
    fn from(
        (pagination, table_name): (Option<PaginationInput<OpaqueCursor<String>>>, &str),
    ) -> Self {
        match pagination {
            Some(pagination) => (pagination, table_name).into(),
            None => PaginationOptions::default(),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct ResultSlice<T> {
    pub results: Vec<T>,
    pub has_previous_page: bool,
    pub has_next_page: bool,
}

impl<T> ResultSlice<T>
where
    T: PartialEq<ID>,
{
    pub fn new(
        results: Vec<T>,
        ResultSliceOptions {
            reverse_results,
            limit,
            after,
            before,
        }: ResultSliceOptions,
    ) -> ResultSlice<T> {
        // Switch into beginning and end terminology because this is wrinckling my brain.
        // From here, we operate on the list as if it's in the correct order, and then
        // reverse it at the end if we need to.
        let (beg_cursor, end_cursor) = if reverse_results {
            (before, after)
        } else {
            (after, before)
        };

        let size = results.len();
        let mut has_more_beg = false;
        let mut has_more_end = false;
        let results: Vec<T> = results
            .into_iter()
            .enumerate()
            .map(|(i, item)| (item, i == 0, i == size - 1))
            .filter_map(|(item, first, last)| {
                let skip = match (first, &beg_cursor) {
                    (true, Some(cursor)) if item == *cursor => {
                        has_more_beg = true;
                        true
                    }
                    _ => false,
                };
                let skip = match (last, &end_cursor) {
                    (true, Some(cursor)) if item == *cursor => {
                        has_more_end = true;
                        true
                    }
                    _ => skip,
                };
                if skip {
                    None
                } else {
                    Some(item)
                }
            })
            // Collect here to make sure that the cursor items are found if they
            // exist in the results.
            .collect::<Vec<_>>()
            .into_iter()
            .take(
                #[allow(clippy::cast_possible_truncation)]
                #[allow(clippy::cast_sign_loss)]
                {
                    limit as usize
                },
            )
            .collect();

        // If the cursor items were found in the results, then those need to
        // be implicitly removed from the size, as those have been accounted
        // for already.
        let size_offset = match (has_more_beg, has_more_end) {
            (true, true) => 2,
            (true, false) | (false, true) => 1,
            (false, false) => 0,
        };

        // If we have still overfetched after taking into account the cursors,
        // then that means that there were more items beyond the query, so pagination
        // is still possible.
        if results.len() < (size - size_offset) {
            has_more_end = true;
        }

        let (results, has_previous_page, has_next_page) = if reverse_results {
            (
                results.into_iter().rev().collect(),
                has_more_end,
                has_more_beg,
            )
        } else {
            (results, has_more_beg, has_more_end)
        };

        ResultSlice {
            results,
            has_previous_page,
            has_next_page,
        }
    }
}

// Basically a copy of the OpaqueCursor type from async-graphql that actually
// impls traits we need.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct OpaqueCursor<T>(pub T);

impl<T> Deref for OpaqueCursor<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for OpaqueCursor<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> CursorType for OpaqueCursor<T>
where
    T: Serialize + DeserializeOwned,
{
    type Error = Error;

    fn decode_cursor(s: &str) -> Result<Self> {
        use base64::prelude::*;
        let data = BASE64_URL_SAFE_NO_PAD.decode(s)?;
        Ok(Self(serde_json::from_slice(&data)?))
    }

    fn encode_cursor(&self) -> String {
        use base64::prelude::*;
        let value = serde_json::to_vec(&self.0).unwrap_or_default();
        BASE64_URL_SAFE_NO_PAD.encode(value)
    }
}
