#[macro_export]
macro_rules! id_obj_impls {
    ($ty:ty) => {
        impl PartialEq for $ty {
            fn eq(&self, other: &Self) -> bool {
                self.id == other.id
            }
        }

        impl Eq for $ty {}

        impl PartialEq<surrealdb::sql::Thing> for $ty {
            fn eq(&self, other: &surrealdb::sql::Thing) -> bool {
                self.id == *other
            }
        }

        impl PartialEq<$ty> for surrealdb::sql::Thing {
            fn eq(&self, other: &$ty) -> bool {
                other.id == *self
            }
        }

        impl PartialEq<ID> for $ty {
            fn eq(&self, other: &ID) -> bool {
                <surrealdb::sql::Thing as $crate::conv::AsMaybeStr>::as_maybe_str(&self.id)
                    == <ID as $crate::conv::AsMaybeStr>::as_maybe_str(other)
            }
        }

        impl PartialEq<$ty> for ID {
            fn eq(&self, other: &$ty) -> bool {
                other.eq(self)
            }
        }
    };
}
