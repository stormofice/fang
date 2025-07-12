CREATE TABLE users
(
    id            INTEGER PRIMARY KEY NOT NULL,
    name          TEXT UNIQUE         NOT NULL,
    password_hash TEXT                NOT NULL
)
