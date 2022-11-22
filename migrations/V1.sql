CREATE TABLE users
(
    id            BIGINT NOT NULL PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    username      TEXT   NOT NULL UNIQUE,
    password_hash TEXT
);