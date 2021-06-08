table! {
    transactions (id) {
        id -> Int4,
        category -> Varchar,
        transactee -> Varchar,
        note -> Nullable<Varchar>,
        transaction_date -> Date,
    }
}
