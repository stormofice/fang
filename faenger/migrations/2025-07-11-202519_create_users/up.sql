CREATE TABLE users
(
    id              INTEGER PRIMARY KEY NOT NULL,
    name            TEXT UNIQUE         NOT NULL,
    password_hash   TEXT                NOT NULL,
    api_key         TEXT                NOT NULL,
    time_registered TEXT                NOT NULL
)
