ALTER TABLE transactions
    DROP COLUMN user_id;
DELETE
FROM users
WHERE id = 'migration'