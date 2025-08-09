CREATE TABLE faenge
(
    id           INTEGER PRIMARY KEY NOT NULL,
    time_created TEXT                NOT NULL,
    title        TEXT,
    url          TEXT                NOT NULL,
    user_id      INTEGER             NOT NULL REFERENCES users (id)
)
