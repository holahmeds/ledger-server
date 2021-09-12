table! {
    transactions (id) {
        id -> Int4,
        category -> Varchar,
        transactee -> Nullable<Varchar>,
        note -> Nullable<Varchar>,
        transaction_date -> Date,
        amount -> Numeric,
    }
}
