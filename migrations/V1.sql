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

CREATE TABLE tweets
(
    id           BIGINT  NOT NULL PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    user_id      BIGINT  NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    text         TEXT,
    like_count   INTEGER NOT NULL DEFAULT 0,
    time_created BIGINT  NOT NULL DEFAULT extract(EPOCH FROM now())
);

CREATE TABLE user_liked_tweets
(
    user_id  BIGINT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    tweet_id BIGINT NOT NULL REFERENCES tweets (id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, tweet_id)
);

CREATE FUNCTION update_like_count()
    RETURNS TRIGGER AS
$$
BEGIN
    IF tg_op = 'INSERT' THEN
        UPDATE tweets SET like_count = like_count + 1 WHERE id = new.tweet_id;
        RETURN new;
    ELSE
        UPDATE tweets SET like_count = like_count - 1 WHERE id = old.tweet_id;
        RETURN old;
    END IF;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_like_count
    AFTER INSERT OR DELETE
    ON user_liked_tweets
    FOR EACH ROW
EXECUTE FUNCTION update_like_count();