// @generated automatically by Diesel CLI.

diesel::table! {
    backlinks (url) {
        url -> Text,
        lobsters_links -> Nullable<Text>,
        hn_links -> Nullable<Text>,
    }
}

diesel::table! {
    faenge (id) {
        id -> Integer,
        time_created -> Text,
        title -> Nullable<Text>,
        url -> Text,
        user_id -> Integer,
        soft_delete -> Bool,
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
    backlinks,
    faenge,
    users,
);
