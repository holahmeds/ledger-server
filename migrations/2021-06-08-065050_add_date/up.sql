ALTER TABLE transactions ADD COLUMN transaction_date date;
UPDATE transactions SET transaction_date = '1970-1-1';
ALTER TABLE transactions ALTER COLUMN transaction_date SET NOT NULL;
