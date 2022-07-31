INSERT INTO users
VALUES ('migration',
        '$argon2i$v=19$m=4096,t=3,p=1$I4qH3srLI2tkzt0ztygLbdIDNOk8wHVsA69RowZtMyY$Ec1ep1PHcuqNggJoLA5EEjNrR8rZX1XvdqgKqGPkiZU');
ALTER TABLE transactions
    ADD COLUMN user_id VARCHAR NOT NULL REFERENCES users (id) DEFAULT 'migration';
ALTER TABLE transactions
    ALTER COLUMN user_id DROP DEFAULT;