CREATE TABLE faenge
(
    id         INTEGER PRIMARY KEY NOT NULL,
    -- Hashed ish url for lookup
    lookup_url TEXT                NOT NULL,
    -- Encrypted data
    data       TEXT                NOT NULL,
    user_id    INTEGER             NOT NULL REFERENCES users (id)
)
