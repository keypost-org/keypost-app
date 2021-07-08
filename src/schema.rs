table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        email -> Varchar,
        psswd_file -> Text,
        deleted -> Bool,
        inserted_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
