table! {
    scopes (name) {
        name -> Varchar,
    }
}

table! {
    users (name) {
        name -> Varchar,
        hash -> Varchar,
        expiration -> Timestamp,
        enabled -> Bool,
        scopes -> Array<Text>,
    }
}

allow_tables_to_appear_in_same_query!(
    scopes,
    users,
);
