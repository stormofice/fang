CREATE TABLE faenge
(
    id           INTEGER PRIMARY KEY NOT NULL,
    url          TEXT                NOT NULL,
    title        TEXT,
    time_created TEXT                NOT NULL,
    user_id      INTEGER             NOT NULL REFERENCES users (id)
)
