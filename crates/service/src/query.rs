use async_graphql::{
    connection::{CursorType, OpaqueCursor},
    ID,
};

use crate::error::{Error, Result};

pub mod srql {
    pub use surrealdb::sql::{statements::*, *};
}

pub const MAX_LIMIT: i64 = 100;
pub const PAGE_EXTRA: i64 = 2;
pub const SRQL_ORDER_ASC: bool = true;
pub const SRQL_ORDER_DESC: bool = false;

pub fn values_table(table: impl Into<String>) -> srql::Values {
    srql::Values(vec![srql::Table(table.into()).into()])
}

pub fn srql_field(field: impl Into<String>) -> srql::Idiom {
    srql::Idiom(vec![srql::Part::Field(srql::Ident(field.into()))])
}

// pub fn srql_param(param: impl Into<String>) -> srql::Param {
//     srql::Param(srql::Ident(param.into()))
// }

pub fn srql_string(str: impl Into<String>) -> srql::Strand {
    srql::Strand(str.into())
}

pub struct PaginationArgs {
    pub after: Option<String>,
    pub before: Option<String>,
    pub first: Option<i32>,
    pub last: Option<i32>,
}

impl PaginationArgs {
    pub fn validate<Cursor: CursorType>(self) -> Result<PaginationInput<Cursor>> {
        self.try_into()
    }
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Default, Clone)]
pub struct PaginationInput<Cursor: CursorType> {
    direction: Option<PaginationDirection>,
    after: Option<Cursor>,
    before: Option<Cursor>,
}

impl<Cursor: CursorType> TryFrom<PaginationArgs> for PaginationInput<Cursor> {
    type Error = Error;

    fn try_from(
        PaginationArgs {
            after,
            before,
            first,
            last,
        }: PaginationArgs,
    ) -> Result<Self> {
        let direction = match (first, last) {
            (Some(_), Some(_)) => {
                return Err(Error::PaginationInvalid(
                    "The \"first\" and \"last\" parameters cannot exist at the same time".into(),
                ))
            }
            (Some(first), None) if first >= 0 => Some(PaginationDirection::First(first as i64)),
            (Some(_), None) => {
                return Err(Error::PaginationInvalid(
                    "The \"first\" parameter must be a non-negative number".into(),
                ))
            }
            (None, Some(last)) if last >= 0 => Some(PaginationDirection::Last(last as i64)),
            (None, Some(_)) => {
                return Err(Error::PaginationInvalid(
                    "The \"last\" parameter must be a non-negative number".into(),
                ))
            }
            (None, None) => None,
        };

        fn parse_cursor<Cursor: CursorType>(cursor: Option<String>) -> Result<Option<Cursor>> {
            cursor
                .map(|cursor| {
                    Cursor::decode_cursor(&cursor)
                        .map_err(|e| Error::PaginationInvalid(e.to_string()))
                })
                .transpose()
        }

        Ok(PaginationInput {
            direction,
            after: parse_cursor(after)?,
            before: parse_cursor(before)?,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResultSliceOptions {
    reverse_results: bool,
    after: Option<ID>,
    before: Option<ID>,
}

#[derive(Debug, Clone, Default)]
pub struct PaginationOptions {
    pub cond: Option<srql::Cond>,
    pub order: Option<srql::Order>,
    pub limit: Option<srql::Limit>,
    pub result_slice_opts: ResultSliceOptions,
}

impl From<PaginationInput<OpaqueCursor<String>>> for PaginationOptions {
    fn from(
        PaginationInput {
            direction,
            after,
            before,
        }: PaginationInput<OpaqueCursor<String>>,
    ) -> Self {
        let (after_expr, after) = after
            .map(|OpaqueCursor(after)| {
                (
                    srql::Expression {
                        l: srql_field("id").into(),
                        o: srql::Operator::LessThanOrEqual,
                        r: srql_string(&after).into(),
                    },
                    ID(after),
                )
            })
            .unzip();

        let (before_expr, before) = before
            .map(|OpaqueCursor(before)| {
                (
                    srql::Expression {
                        l: srql_field("id").into(),
                        o: srql::Operator::MoreThanOrEqual,
                        r: srql_string(&before).into(),
                    },
                    ID(before),
                )
            })
            .unzip();

        PaginationOptions {
            cond: match (after_expr, before_expr) {
                (Some(after), Some(before)) => srql::Cond(
                    srql::Expression {
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
            order: Some(srql::Order {
                order: srql_field("id"),
                direction: match direction {
                    Some(PaginationDirection::Last(_)) => SRQL_ORDER_ASC,
                    _ => SRQL_ORDER_DESC,
                },
                ..Default::default()
            }),
            limit: Some(srql::Limit(
                srql::Number::Int(
                    std::cmp::max(
                        direction
                            .as_ref()
                            .map(PaginationDirection::limit)
                            .unwrap_or(MAX_LIMIT),
                        MAX_LIMIT,
                    ) + PAGE_EXTRA,
                )
                .into(),
            )),
            result_slice_opts: ResultSliceOptions {
                reverse_results: match direction {
                    Some(PaginationDirection::Last(_)) => true,
                    _ => false,
                },
                after,
                before,
            },
        }
    }
}

impl From<Option<PaginationInput<OpaqueCursor<String>>>> for PaginationOptions {
    fn from(pagination: Option<PaginationInput<OpaqueCursor<String>>>) -> Self {
        match pagination {
            Some(pagination) => pagination.into(),
            None => Default::default(),
        }
    }
}

#[derive(Debug, Default, Clone)]
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
            after,
            before,
        }: ResultSliceOptions,
    ) -> ResultSlice<T> {
        let size = results.len();
        let has_previous_page = match after {
            Some(after) => match results.first() {
                Some(first) if *first == after => true,
                _ => false,
            },
            None => false,
        };

        let has_next_page = match before {
            Some(before) => match results.last() {
                Some(last) if *last == before => true,
                _ => false,
            },
            None => false,
        };

        let results = if reverse_results {
            results.into_iter().rev().collect()
        } else {
            results
        };

        let mut res = Vec::with_capacity(size);
        for (i, result) in results.into_iter().enumerate() {
            if (i == 0 && has_previous_page) || (i == size - 1 && has_next_page) {
                continue;
            }
            res.push(result);
        }

        Self {
            results: res,
            has_previous_page,
            has_next_page,
        }
    }
}
