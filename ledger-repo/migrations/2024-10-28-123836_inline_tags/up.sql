ALTER TABLE transactions
    ADD COLUMN tags text[] NOT NULL DEFAULT '{}';

UPDATE transactions
SET tags = COALESCE((SELECT ARRAY_AGG(tag) FROM transaction_tags WHERE transaction_id = id), '{}');
