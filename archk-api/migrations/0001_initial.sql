CREATE TABLE users (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    invites INTEGER NOT NULL DEFAULT 0,
    invited_by TEXT DEFAULT NULL,

    level INTEGER NOT NULL DEFAULT 0,

    password_hash TEXT NOT NULL,

    FOREIGN KEY(invited_by) REFERENCES users(id) ON DELETE SET NULL
);

CREATE TABLE invites (
    id TEXT NOT NULL PRIMARY KEY,
    owner_id TEXT DEFAULT NULL,

    FOREIGN KEY(owner_id) REFERENCES users(id) ON DELETE SET NULL
);

CREATE TABLE tokens (
    iat INTEGER NOT NULL,
    rnd INTEGER NOT NULL,

    user_id TEXT NOT NULL,

    PRIMARY KEY(iat, rnd),
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE spaces (
    id TEXT NOT NULL PRIMARY KEY,
    title TEXT NOT NULL,
    owner_id TEXT NOT NULL,

    FOREIGN KEY(owner_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE spaces_accounts (
    pl_id TEXT NOT NULL,
    space_id TEXT NOT NULL,

    pl_name TEXT DEFAULT NULL,
    pl_displayname TEXT DEFAULT NULL,

    PRIMARY KEY(space_id, pl_id),
    FOREIGN KEY(space_id) REFERENCES spaces(id) ON DELETE CASCADE
);

CREATE TABLE spaces_items (
    id TEXT NOT NULL PRIMARY KEY,
    title TEXT NOT NULL,
    ty INTEGER NOT NULL DEFAULT 0,

    pl_serial TEXT NOT NULL,

    owner_id TEXT DEFAULT NULL,
    space_id TEXT NOT NULL,
    
    UNIQUE(pl_serial, space_id),
    FOREIGN KEY(owner_id, space_id) REFERENCES spaces_accounts(pl_id, space_id) ON DELETE CASCADE,
    FOREIGN KEY(space_id) REFERENCES spaces(id) ON DELETE CASCADE
);

CREATE TABLE spaces_logs (
    id TEXT NOT NULL PRIMARY KEY,
    space_id TEXT NOT NULL,
    created_at INTEGER NOT NULL,

    act INTEGER NOT NULL,
    sp_acc_id TEXT DEFAULT NULL,
    sp_item_id TEXT DEFAULT NULL,

    -- oh please ignore it
    -- FOREIGN KEY(sp_item_id) REFERENCES spaces_accounts(id) ON DELETE SET NULL,
    -- FOREIGN KEY(sp_acc_id, space_id) REFERENCES spaces_accounts(pl_id, space_id) ON DELETE SET NULL,
    FOREIGN KEY(space_id) REFERENCES spaces(id) ON DELETE CASCADE
);
