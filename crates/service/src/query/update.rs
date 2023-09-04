use async_graphql::MaybeUndefined;

use super::srql;

pub type SetExpr = Vec<(srql::Idiom, srql::Operator, srql::Value)>;

pub trait IntoUpdateQuery {
    fn into_update(self, thing: srql::Thing) -> Option<srql::Query>;

    fn update_query(thing: srql::Thing, mut update: SetExpr) -> Option<srql::Query> {
        if update.is_empty() {
            return None;
        }

        update.push((
            srql::field("updated_at"),
            srql::Operator::Equal,
            srql::Value::Function(Box::new(srql::Function::Normal("time::now".into(), vec![]))),
        ));

        srql::Query(srql::Statements(vec![srql::Statement::Update(
            srql::UpdateStatement {
                what: srql::Values(vec![thing.into()]),
                data: srql::Data::UpdateExpression(update).into(),
                output: srql::Output::After.into(),
                ..Default::default()
            },
        )]))
        .into()
    }
}

pub trait QueryUpdateValue {
    fn apply_update(self, field: srql::Idiom, update: &mut SetExpr);
}

impl QueryUpdateValue for String {
    fn apply_update(self, field: srql::Idiom, update: &mut SetExpr) {
        update.push((
            field,
            srql::Operator::Equal,
            srql::Value::Strand(self.into()),
        ));
    }
}

impl<T: QueryUpdateValue> QueryUpdateValue for Option<T> {
    fn apply_update(self, field: srql::Idiom, update: &mut SetExpr) {
        if let Some(v) = self {
            v.apply_update(field, update);
        }
    }
}

impl<T: QueryUpdateValue> QueryUpdateValue for MaybeUndefined<T> {
    fn apply_update(self, field: srql::Idiom, update: &mut SetExpr) {
        use MaybeUndefined as E;
        match self {
            E::Value(v) => v.apply_update(field, update),
            E::Null => update.push((field, srql::Operator::Equal, srql::Value::None)),
            E::Undefined => (),
        }
    }
}
