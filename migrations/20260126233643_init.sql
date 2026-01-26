CREATE TABLE IF NOT EXISTS locations
(
    id          INTEGER PRIMARY KEY NOT NULL,
    latitude    TEXT                NOT NULL,
    longitude   TEXT                NOT NULL,
    timestamp   DATETIME            NOT NULL
);
