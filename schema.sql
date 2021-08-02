DROP TABLE IF EXISTS users;
CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL UNIQUE,
    is_admin BOOL NOT NULL DEFAULT false,
    is_approved BOOL NOT NULL DEFAULT false
);
CREATE UNIQUE INDEX uname ON users(username);

DROP TABLE IF EXISTS items;
CREATE TABLE items (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    html TEXT NOT NULL,
    markdown TEXT NOT NULL,
    discussed_on DATE DEFAULT NULL
);

DROP TABLE IF EXISTS votes;
CREATE TABLE votes (
    user_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    ordinal INTEGER NOT NULL,

    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
    FOREIGN KEY(item_id) REFERENCES items(id) ON DELETE CASCADE
);
CREATE UNIQUE INDEX no_dup_votes ON votes(user_id, item_id);
CREATE INDEX ballot ON votes(user_id ASC, ordinal ASC);
