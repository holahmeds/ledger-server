CREATE TABLE transaction_tags (
    transaction_id SERIAL REFERENCES transactions(id) ON DELETE CASCADE,
    tag VARCHAR NOT NULL,
    PRIMARY KEY(transaction_id, tag)
)