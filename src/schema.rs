table! {
    lockers (id) {
        id -> Int4,
        email -> Varchar,
        locker_id -> Varchar,
        psswd_file -> Text,
        ciphertext -> Text,
        inserted_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

table! {
    users (id) {
        id -> Int4,
        email -> Varchar,
        psswd_file -> Text,
        deleted -> Bool,
        inserted_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

allow_tables_to_appear_in_same_query!(lockers, users,);
