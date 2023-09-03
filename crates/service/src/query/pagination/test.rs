use test_case::test_case;

use super::{testing::*, *};

#[test]
fn test_pagination_direction_limit() {
    let first_direction = PaginationDirection::First(10);
    assert_eq!(first_direction.limit(), 10);

    let last_direction = PaginationDirection::Last(5);
    assert_eq!(last_direction.limit(), 5);
}

#[test_case(
    PaginationArgs {
        after: Some(encoded_cursor("abc")),
        before: None,
        first: Some(10),
        last: None,
    } =>
    Ok(PaginationInput {
        direction: Some(PaginationDirection::First(10)),
        after: Some(OpaqueCursor("abc".to_string())),
        before: None,
    });
    "after & first"
)]
#[test_case(
    PaginationArgs {
        after: None,
        before: Some(encoded_cursor("def")),
        first: None,
        last: Some(5),
    } =>
    Ok(PaginationInput {
        direction: Some(PaginationDirection::Last(5)),
        after: None,
        before: Some(OpaqueCursor("def".to_string())),
    });
    "before & last"
)]
#[test_case(
    PaginationArgs {
        after: Some(encoded_cursor("abc")),
        before: Some(encoded_cursor("def")),
        first: None,
        last: None,
    } =>
    Ok(PaginationInput {
        direction: None,
        after: Some(OpaqueCursor("abc".to_string())),
        before: Some(OpaqueCursor("def".to_string())),
    });
    "after & before"
)]
#[test_case(
    PaginationArgs {
        after: None,
        before: None,
        first: None,
        last: None,
    } =>
    Ok(PaginationInput {
        direction: None,
        after: None,
        before: None,
    });
    "none"
)]
#[test_case(
    PaginationArgs {
        after: Some(encoded_cursor("abc")),
        before: Some(encoded_cursor("def")),
        first: Some(10),
        last: None,
    } =>
    Ok(PaginationInput {
        direction: Some(PaginationDirection::First(10)),
        after: Some(OpaqueCursor("abc".to_string())),
        before: Some(OpaqueCursor("def".to_string())),
    });
    "after & before & first"
)]
fn test_pagination_input_try_from(
    input: PaginationArgs,
) -> Result<PaginationInput<OpaqueCursor<String>>> {
    let result: PaginationInput<OpaqueCursor<String>> = input.try_into()?;
    println!("{result:?}");
    Ok(result)
}

#[test_case(
    PaginationArgs {
        after: Some(encoded_cursor("abc")),
        before: Some(encoded_cursor("def")),
        first: Some(10),
        last: Some(5),
    };
    "after & before & first & last"
)]
#[test_case(
    PaginationArgs {
        after: None,
        before: None,
        first: Some(10),
        last: Some(5),
    };
    "first & last"
)]
#[test_case(
    PaginationArgs {
        after: None,
        before: None,
        first: Some(-10),
        last: None,
    };
    "first < 0"
)]
#[test_case(
    PaginationArgs {
        after: None,
        before: None,
        first: None,
        last: Some(-5),
    };
    "last < 0"
)]
fn test_pagination_input_try_from_err(input: PaginationArgs) {
    let result: Result<PaginationInput<OpaqueCursor<String>>> = input.try_into();
    println!("{result:?}");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::PaginationInvalid(_)));
}

static TABLE_NAME: &str = "test_table";

#[test_case(
    PaginationInput {
        direction: Some(PaginationDirection::First(10)),
        after: Some(OpaqueCursor("abc".to_string())),
        before: None,
    } =>
    PaginationOptions {
        cond: Some(srql::Cond(
            srql::Expression::Binary {
                l: srql_field("id").into(),
                o: srql::Operator::LessThanOrEqual,
                r: srql::Thing::from((TABLE_NAME, "abc")).into(),
            }
            .into()
        )),
        order: Some(srql::Order {
            order: srql_field("id"),
            direction: SRQL_ORDER_DESC,
            ..Default::default()
        }),
        limit: Some(srql::Limit(srql::Number::Int(12).into())),
        result_slice_opts: ResultSliceOptions {
            reverse_results: false,
            limit: 10,
            after: Some(ID("abc".to_string())),
            before: None,
        },
    };
    "first & after"
)]
#[test_case(
    PaginationInput {
        direction: Some(PaginationDirection::Last(5)),
        after: None,
        before: Some(OpaqueCursor("def".to_string())),
    } =>
    PaginationOptions {
        cond: Some(srql::Cond(
            srql::Expression::Binary {
                l: srql_field("id").into(),
                o: srql::Operator::MoreThanOrEqual,
                r: srql::Thing::from((TABLE_NAME, "def")).into(),
            }
            .into()
        )),
        order: Some(srql::Order {
            order: srql_field("id"),
            direction: SRQL_ORDER_ASC,
            ..Default::default()
        }),
        limit: Some(srql::Limit(srql::Number::Int(7).into())),
        result_slice_opts: ResultSliceOptions {
            reverse_results: true,
            limit: 5,
            after: None,
            before: Some(ID("def".to_string())),
        },
    };
    "last & before"
)]
#[test_case(
    PaginationInput {
        direction: Some(PaginationDirection::Last(111)),
        after: None,
        before: Some(OpaqueCursor("def".to_string())),
    } =>
    PaginationOptions {
        cond: Some(srql::Cond(
            srql::Expression::Binary {
                l: srql_field("id").into(),
                o: srql::Operator::MoreThanOrEqual,
                r: srql::Thing::from((TABLE_NAME, "def")).into(),
            }
            .into()
        )),
        order: Some(srql::Order {
            order: srql_field("id"),
            direction: SRQL_ORDER_ASC,
            ..Default::default()
        }),
        limit: Some(srql::Limit(srql::Number::Int(MAX_LIMIT + PAGE_EXTRA).into())),
        result_slice_opts: ResultSliceOptions {
            reverse_results: true,
            limit: MAX_LIMIT,
            after: None,
            before: Some(ID("def".to_string())),
        },
    };
    "last & before & limit > MAX_LIMIT"
)]
#[test_case(
    PaginationInput {
        direction: None,
        after: Some(OpaqueCursor("abc".to_string())),
        before: Some(OpaqueCursor("def".to_string())),
    } =>
    PaginationOptions {
        cond: Some(srql::Cond(
            srql::Expression::Binary {
                l: srql::Expression::Binary {
                    l: srql_field("id").into(),
                    o: srql::Operator::LessThanOrEqual,
                    r: srql::Thing::from((TABLE_NAME, "abc")).into()
                }
                .into(),
                o: srql::Operator::And,
                r: srql::Expression::Binary {
                    l: srql_field("id").into(),
                    o: srql::Operator::MoreThanOrEqual,
                    r: srql::Thing::from((TABLE_NAME, "def")).into()
                }
                .into(),
            }
            .into()
        )),
        order: Some(srql::Order {
            order: srql_field("id"),
            direction: SRQL_ORDER_DESC,
            ..Default::default()
        }),
        limit: Some(srql::Limit(srql::Number::Int(MAX_LIMIT + PAGE_EXTRA).into())),
        result_slice_opts: ResultSliceOptions {
            reverse_results: false,
            limit: MAX_LIMIT,
            after: Some(ID("abc".to_string())),
            before: Some(ID("def".to_string())),
        },
    };
    "after & before"
)]
fn test_pagination_options_from(input: PaginationInput<OpaqueCursor<String>>) -> PaginationOptions {
    (input, TABLE_NAME).into()
}

fn id(i: i64) -> ID {
    ID(i.to_string())
}

const ID_COUNT: i64 = 10;

fn ids() -> Vec<ID> {
    (0..ID_COUNT).map(id).collect()
}

#[allow(clippy::cast_possible_truncation)]
#[test_case(
    ResultSliceOptions {
        reverse_results: false,
        limit: ID_COUNT,
        after: None,
        before: None,
    } =>
    ResultSlice {
        results: ids(),
        has_previous_page: false,
        has_next_page: false,
    };
    "no cursor"
)]
#[test_case(
    ResultSliceOptions {
        reverse_results: false,
        limit: ID_COUNT - 1,
        after: Some(id(0)),
        before: None,
    } =>
    ResultSlice {
        results: ids()[1..].to_vec(),
        has_previous_page: true,
        has_next_page: false,
    };
    "after cursor"
)]
#[test_case(
    ResultSliceOptions {
        reverse_results: false,
        limit: ID_COUNT - 1,
        after: None,
        before: Some(id(ID_COUNT - 1)),
    } =>
    ResultSlice {
        results: ids()[..(ID_COUNT - 1) as usize].to_vec(),
        has_previous_page: false,
        has_next_page: true,
    };
    "before cursor"
)]
#[test_case(
    ResultSliceOptions {
        reverse_results: false,
        limit: ID_COUNT - 2,
        after: Some(id(0)),
        before: Some(id(ID_COUNT - 1)),
    } =>
    ResultSlice {
        results: ids()[1..(ID_COUNT - 1) as usize].to_vec(),
        has_previous_page: true,
        has_next_page: true,
    };
    "after & before cursor"
)]
#[test_case(
    ResultSliceOptions {
        reverse_results: true,
        limit: ID_COUNT,
        after: None,
        before: None,
    } =>
    ResultSlice {
        results: ids(),
        has_previous_page: false,
        has_next_page: false,
    };
    "reverse results"
)]
#[test_case(
    ResultSliceOptions {
        reverse_results: true,
        limit: ID_COUNT - 1,
        after: Some(id(0)),
        before: None,
    } =>
    ResultSlice {
        results: ids()[1..].to_vec(),
        has_previous_page: true,
        has_next_page: false,
    };
    "reverse results & after cursor"
)]
#[test_case(
    ResultSliceOptions {
        reverse_results: true,
        limit: ID_COUNT - 1,
        after: None,
        before: Some(id(ID_COUNT - 1)),
    } =>
    ResultSlice {
        results: ids()[..(ID_COUNT - 1) as usize].to_vec(),
        has_previous_page: false,
        has_next_page: true,
    };
    "reverse results & before cursor"
)]
#[test_case(
    ResultSliceOptions {
        reverse_results: true,
        limit: ID_COUNT - 2,
        after: Some(id(0)),
        before: Some(id(ID_COUNT - 1)),
    } =>
    ResultSlice {
        results: ids()[1..(ID_COUNT - 1) as usize].to_vec(),
        has_previous_page: true,
        has_next_page: true,
    };
    "reverse results & after & before cursor"
)]
#[test_case(
    ResultSliceOptions {
        reverse_results: false,
        limit: ID_COUNT - PAGE_EXTRA,
        after: None,
        before: None,
    } =>
    ResultSlice {
        results: ids()[..(ID_COUNT - PAGE_EXTRA) as usize].to_vec(),
        has_previous_page: false,
        has_next_page: true,
    };
    "no cursor & limit < size"
)]
#[test_case(
    ResultSliceOptions {
        reverse_results: true,
        limit: ID_COUNT - PAGE_EXTRA,
        after: None,
        before: None,
    } =>
    ResultSlice {
        results: ids()[PAGE_EXTRA as usize..].to_vec(),
        has_previous_page: true,
        has_next_page: false,
    };
    "no cursor & limit < size & reverse results"
)]
fn test_result_slice_new(opts: ResultSliceOptions) -> ResultSlice<ID> {
    ResultSlice::new(
        // When reverse_results is true, it's expected that the input results
        // are already reversed, and that we need to de-reverse them to match
        // the cursor spec.
        if opts.reverse_results {
            ids().into_iter().rev().collect()
        } else {
            ids()
        },
        opts,
    )
}
