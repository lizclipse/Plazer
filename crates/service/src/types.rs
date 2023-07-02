#[derive(Debug, Default, Clone)]
pub struct PaginationArgs {
    after: Option<String>,
    before: Option<String>,
    first: Option<i32>,
    last: Option<i32>,
}

pub type PaginationTuple = (Option<String>, Option<String>, Option<i32>, Option<i32>);

impl From<PaginationTuple> for PaginationArgs {
    fn from((after, before, first, last): PaginationTuple) -> Self {
        Self {
            after,
            before,
            first,
            last,
        }
    }
}

impl From<PaginationArgs> for PaginationTuple {
    fn from(args: PaginationArgs) -> Self {
        (args.after, args.before, args.first, args.last)
    }
}

pub trait IntoPaginationTuple {
    fn into_pagination_tuple(self) -> PaginationTuple;
}

impl IntoPaginationTuple for Option<PaginationArgs> {
    fn into_pagination_tuple(self) -> PaginationTuple {
        match self {
            Some(args) => args.into(),
            None => Default::default(),
        }
    }
}
