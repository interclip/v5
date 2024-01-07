// @generated automatically by Diesel CLI.

diesel::table! {
    clips (id) {
        id -> Nullable<Integer>,
        code -> Text,
        url -> Text,
        date -> Timestamp,
        expires -> Date,
    }
}
