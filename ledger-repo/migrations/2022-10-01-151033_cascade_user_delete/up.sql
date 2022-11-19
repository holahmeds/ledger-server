ALTER TABLE transactions
    DROP CONSTRAINT transactions_user_id_fkey,
    ADD CONSTRAINT transactions_user_id_fkey FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE;
