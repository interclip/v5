// @generated automatically by Diesel CLI.

diesel::table! {
    clips (id) {
        id -> Int4,
        url -> Text,
        code -> Text,
        created_at -> Timestamp,
        expires_at -> Nullable<Timestamp>,
    }
}
