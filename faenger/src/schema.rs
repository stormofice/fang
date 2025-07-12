// @generated automatically by Diesel CLI.

diesel::table! {
    faenge (id) {
        id -> Integer,
        url -> Text,
        title -> Nullable<Text>,
        time_created -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
        password_hash -> Text,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    faenge,
    users,
);
