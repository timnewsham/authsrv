table! {
    scopes (name) {
        name -> Varchar,
    }
}

table! {
    tokens (token) {
        token -> Varchar,
        username -> Varchar,
        expiration -> Timestamp,
        scopes -> Array<Text>,
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
    tokens,
    users,
);
