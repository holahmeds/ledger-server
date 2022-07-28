table! {
    transaction_tags (transaction_id, tag) {
        transaction_id -> Int4,
        tag -> Varchar,
    }
}

table! {
    transactions (id) {
        id -> Int4,
        category -> Varchar,
        transactee -> Nullable<Varchar>,
        note -> Nullable<Varchar>,
        date -> Date,
        amount -> Numeric,
    }
}

table! {
    users (id) {
        id -> Varchar,
        password_hash -> Varchar,
    }
}

joinable!(transaction_tags -> transactions (transaction_id));

allow_tables_to_appear_in_same_query!(
    transaction_tags,
    transactions,
    users,
);
