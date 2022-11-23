CREATE TABLE users
(
    id            BIGINT NOT NULL PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    username      TEXT   NOT NULL UNIQUE,
    password_hash TEXT
);

CREATE TABLE sessions
(
    id      BIGINT NOT NULL PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    user_id BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE
--  We can store the user agent and/or ip address here and then compare them.
--  If they do not match - the session will be invalidated.
--
--  This way of session management is also useful if we want to let
--  our users see the list of their active sessions.
--
--  Security™
);