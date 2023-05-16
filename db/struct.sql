CREATE TABLE users (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    email     TEXT    UNIQUE,
    access    INTEGER,
    discordid INTEGER UNIQUE
);

CREATE TABLE pools (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT,
    motp TEXT
);

CREATE TABLE poolers (
    id      INTEGER PRIMARY KEY AUTOINCREMENT,
    name    TEXT,
    favteam TEXT,
    poolid  INTEGER CONSTRAINT PoolId_FK REFERENCES pools (id) ON DELETE SET NULL,
    userid  INTEGER CONSTRAINT UserId_FK REFERENCES users (id) ON DELETE SET NULL
);

CREATE TABLE picks (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    season     INTEGER,
    week       INTEGER,
    pickstring TEXT,
    poolerid           CONSTRAINT PoolerId_FK REFERENCES poolers (id) ON DELETE SET NULL
);
