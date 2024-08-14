CREATE TABLE users_ssh_keys (
    id TEXT NOT NULL PRIMARY KEY,
    pubkey_ty INTEGER NOT NULL,
    pubkey_val TEXT NOT NULL,
    pubkey_fingerprint TEXT NOT NULL,
    owner_id TEXT NOT NULL,

    UNIQUE(pubkey_ty, pubkey_val),
    FOREIGN KEY(owner_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE INDEX idx_users_ssh_keys_fingerprint ON users_ssh_keys(pubkey_fingerprint);

CREATE TABLE service_accounts (
    id TEXT NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    space_id TEXT DEFAULT NULL,
    ty INTEGER NOT NULL,

    FOREIGN KEY(space_id) REFERENCES spaces(id) ON DELETE CASCADE
);

CREATE TABLE service_tokens (
    iat INTEGER NOT NULL,
    rnd INTEGER NOT NULL,

    service_id TEXT NOT NULL,

    PRIMARY KEY(iat, rnd),
    FOREIGN KEY(service_id) REFERENCES service_accounts(id) ON DELETE CASCADE
);
