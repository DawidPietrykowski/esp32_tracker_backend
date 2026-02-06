CREATE TABLE IF NOT EXISTS locations
(
    id          INTEGER PRIMARY KEY NOT NULL,
    latitude    REAL                NOT NULL,
    longitude   REAL                NOT NULL,
    signal      REAL                NOT NULL,
    battery     REAL                NOT NULL,
    generated   DATETIME            NOT NULL,
    received    DATETIME            NOT NULL
);
