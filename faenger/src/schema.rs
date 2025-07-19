// @generated automatically by Diesel CLI.

diesel::table! {
    faenge (id) {
        id -> Integer,
        url -> Text,
        title -> Nullable<Text>,
        time_created -> Text,
        user_id -> Integer,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        name -> Text,
        password_hash -> Text,
        api_key -> Text,
        time_registered -> Text,
    }
}

diesel::joinable!(faenge -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    faenge,
    users,
);
