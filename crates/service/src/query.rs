use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use async_graphql::{connection::CursorType, ID};
use serde::{de::DeserializeOwned, Serialize};

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
                    std::cmp::min(
                        direction
                            .as_ref()
                            .map_or(MAX_LIMIT, PaginationDirection::limit),
                        MAX_LIMIT,
                    ) + PAGE_EXTRA,
                )
                .into(),
            )),
            result_slice_opts: ResultSliceOptions {
                reverse_results: matches!(direction, Some(PaginationDirection::Last(_))),
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
            after,
            before,
        }: ResultSliceOptions,
    ) -> ResultSlice<T> {
        let size = results.len();
        let has_previous_page = match after {
            Some(after) => matches!(results.first(), Some(first) if *first == after),
            None => false,
        };

        let has_next_page = match before {
            Some(before) => matches!(results.last(), Some(last) if *last == before),
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

#[cfg(test)]
mod tests {
    use super::{testing::*, *};

    #[test]
    fn test_pagination_direction_limit() {
        let first_direction = PaginationDirection::First(10);
        assert_eq!(first_direction.limit(), 10);

        let last_direction = PaginationDirection::Last(5);
        assert_eq!(last_direction.limit(), 5);
    }

    #[test]
    fn test_pagination_input_try_from() {
        let pagination_args = PaginationArgs {
            after: Some(encoded_cursor("abc")),
            before: None,
            first: Some(10),
            last: None,
        };
        let pagination_input: PaginationInput<OpaqueCursor<String>> =
            pagination_args.validate().unwrap();
        println!("{pagination_input:?}");
        assert_eq!(
            pagination_input.direction,
            Some(PaginationDirection::First(10))
        );
        assert_eq!(
            pagination_input.after,
            Some(OpaqueCursor("abc".to_string()))
        );
        assert_eq!(pagination_input.before, None);

        let pagination_args = PaginationArgs {
            after: None,
            before: Some(encoded_cursor("def")),
            first: None,
            last: Some(5),
        };
        let pagination_input: PaginationInput<OpaqueCursor<String>> =
            pagination_args.validate().unwrap();
        println!("{pagination_input:?}");
        assert_eq!(
            pagination_input.direction,
            Some(PaginationDirection::Last(5))
        );
        assert_eq!(pagination_input.after, None);
        assert_eq!(
            pagination_input.before,
            Some(OpaqueCursor("def".to_string()))
        );

        let pagination_args = PaginationArgs {
            after: None,
            before: None,
            first: None,
            last: None,
        };
        let pagination_input: PaginationInput<OpaqueCursor<String>> =
            pagination_args.validate().unwrap();
        println!("{pagination_input:?}");
        assert_eq!(pagination_input.direction, None);
        assert_eq!(pagination_input.after, None);
        assert_eq!(pagination_input.before, None);

        let pagination_args = PaginationArgs {
            after: Some(encoded_cursor("abc")),
            before: Some(encoded_cursor("def")),
            first: Some(10),
            last: Some(5),
        };
        let result: Result<PaginationInput<OpaqueCursor<String>>> = pagination_args.validate();
        println!("{result:?}");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::PaginationInvalid(_)));

        let pagination_args = PaginationArgs {
            after: Some(encoded_cursor("abc")),
            before: Some(encoded_cursor("def")),
            first: Some(10),
            last: None,
        };
        let pagination_input: PaginationInput<OpaqueCursor<String>> =
            pagination_args.validate().unwrap();
        println!("{pagination_input:?}");
        assert_eq!(
            pagination_input.direction,
            Some(PaginationDirection::First(10))
        );
        assert_eq!(
            pagination_input.after,
            Some(OpaqueCursor("abc".to_string()))
        );
        assert_eq!(
            pagination_input.before,
            Some(OpaqueCursor("def".to_string()))
        );

        let pagination_args = PaginationArgs {
            after: None,
            before: None,
            first: Some(10),
            last: Some(5),
        };
        let result: Result<PaginationInput<OpaqueCursor<String>>> = pagination_args.validate();
        println!("{result:?}");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::PaginationInvalid(_)));

        let pagination_args = PaginationArgs {
            after: None,
            before: None,
            first: Some(-10),
            last: None,
        };
        let result: Result<PaginationInput<OpaqueCursor<String>>> = pagination_args.validate();
        println!("{result:?}");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::PaginationInvalid(_)));

        let pagination_args = PaginationArgs {
            after: None,
            before: None,
            first: None,
            last: Some(-5),
        };
        let result: Result<PaginationInput<OpaqueCursor<String>>> = pagination_args.validate();
        println!("{result:?}");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::PaginationInvalid(_)));
    }

    #[test]
    fn test_pagination_options_from() {
        let pagination_input = PaginationInput {
            direction: Some(PaginationDirection::First(10)),
            after: Some(OpaqueCursor("abc".to_string())),
            before: None,
        };
        let pagination_options: PaginationOptions = pagination_input.into();
        println!("{pagination_options:?}");
        assert_eq!(
            pagination_options.cond,
            Some(srql::Cond(
                srql::Expression {
                    l: srql_field("id").into(),
                    o: srql::Operator::LessThanOrEqual,
                    r: srql_string("abc").into(),
                }
                .into()
            ))
        );
        assert_eq!(
            pagination_options.order,
            Some(srql::Order {
                order: srql_field("id"),
                direction: SRQL_ORDER_DESC,
                ..Default::default()
            })
        );
        assert_eq!(
            pagination_options.limit,
            Some(srql::Limit(srql::Number::Int(12).into()))
        );
        assert_eq!(
            pagination_options.result_slice_opts.after,
            Some(ID("abc".to_string()))
        );
        assert_eq!(pagination_options.result_slice_opts.before, None);
        assert!(!pagination_options.result_slice_opts.reverse_results);

        let pagination_input = PaginationInput {
            direction: Some(PaginationDirection::Last(5)),
            after: None,
            before: Some(OpaqueCursor("def".to_string())),
        };
        let pagination_options: PaginationOptions = pagination_input.into();
        println!("{pagination_options:?}");
        assert_eq!(
            pagination_options.cond,
            Some(srql::Cond(
                srql::Expression {
                    l: srql_field("id").into(),
                    o: srql::Operator::MoreThanOrEqual,
                    r: srql_string("def").into(),
                }
                .into()
            ))
        );
        assert_eq!(
            pagination_options.order,
            Some(srql::Order {
                order: srql_field("id"),
                direction: SRQL_ORDER_ASC,
                ..Default::default()
            })
        );
        assert_eq!(
            pagination_options.limit,
            Some(srql::Limit(srql::Number::Int(7).into()))
        );
        assert_eq!(pagination_options.result_slice_opts.after, None);
        assert_eq!(
            pagination_options.result_slice_opts.before,
            Some(ID("def".to_string()))
        );
        assert!(pagination_options.result_slice_opts.reverse_results);

        let pagination_input = PaginationInput {
            direction: Some(PaginationDirection::Last(111)),
            after: None,
            before: Some(OpaqueCursor("def".to_string())),
        };
        let pagination_options: PaginationOptions = pagination_input.into();
        println!("{pagination_options:?}");
        assert_eq!(
            pagination_options.limit,
            Some(srql::Limit(srql::Number::Int(102).into()))
        );

        let pagination_input = PaginationInput {
            direction: None,
            after: Some(OpaqueCursor("abc".to_string())),
            before: Some(OpaqueCursor("def".to_string())),
        };
        let pagination_options: PaginationOptions = pagination_input.into();
        println!("{pagination_options:?}");
        assert_eq!(
            pagination_options.cond,
            Some(srql::Cond(
                srql::Expression {
                    l: srql::Expression {
                        l: srql_field("id").into(),
                        o: srql::Operator::LessThanOrEqual,
                        r: srql_string("abc").into(),
                    }
                    .into(),
                    o: srql::Operator::And,
                    r: srql::Expression {
                        l: srql_field("id").into(),
                        o: srql::Operator::MoreThanOrEqual,
                        r: srql_string("def").into(),
                    }
                    .into(),
                }
                .into()
            ))
        );
    }
}

#[cfg(test)]
pub mod testing {
    use super::*;

    pub fn encoded_cursor(id: impl Into<String>) -> String {
        OpaqueCursor(id.into()).encode_cursor()
    }
}
