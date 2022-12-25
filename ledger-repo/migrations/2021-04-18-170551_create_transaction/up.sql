CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    category VARCHAR NOT NULL,
    transactee VARCHAR NOT NULL,
    note VARCHAR
)