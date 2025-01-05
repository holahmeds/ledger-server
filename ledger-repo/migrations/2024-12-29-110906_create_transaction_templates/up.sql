CREATE TABLE transaction_templates
(
    template_id SERIAL PRIMARY KEY,
    category    VARCHAR,
    transactee  VARCHAR,
    note        VARCHAR,
    amount      NUMERIC,
    user_id     VARCHAR             NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    tags        TEXT[] DEFAULT '{}' NOT NULL
);
