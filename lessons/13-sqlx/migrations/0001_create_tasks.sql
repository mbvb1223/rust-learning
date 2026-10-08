CREATE TABLE tasks (
    -- AUTOINCREMENT: the id of a deleted task is never handed out again.
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    -- Backstop for the Rust validation. SQLite's length() counts characters, not bytes.
    title     TEXT    NOT NULL CHECK (length(title) BETWEEN 1 AND 200),
    -- SQLite has no boolean storage class; SQLx maps Rust `bool` to 0/1.
    completed INTEGER NOT NULL DEFAULT 0 CHECK (completed IN (0, 1))
);
