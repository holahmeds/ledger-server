ALTER TABLE transactions ADD COLUMN amount numeric;
UPDATE transactions SET amount = 0;
ALTER TABLE transactions ALTER COLUMN amount SET NOT NULL;
