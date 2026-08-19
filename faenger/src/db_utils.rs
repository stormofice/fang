use diesel::define_sql_function;
use diesel::sql_types::{Nullable, Text};

define_sql_function! { fn coalesce(x: Nullable<Text>, y: Nullable<Text>) -> Nullable<Text>; }
