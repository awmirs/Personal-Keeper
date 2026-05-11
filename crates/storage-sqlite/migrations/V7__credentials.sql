CREATE TABLE IF NOT EXISTS credentials_config (
                                                  id TEXT PRIMARY KEY DEFAULT 'master',
                                                  password_hash TEXT NOT NULL,
                                                  salt BLOB NOT NULL
);

CREATE TABLE IF NOT EXISTS credentials (
                                           id TEXT PRIMARY KEY,
                                           website TEXT NOT NULL,
                                           url TEXT NOT NULL DEFAULT '',
                                           username TEXT NOT NULL,
                                           password_encrypted TEXT,   -- JSON of EncryptedData
                                           notes_encrypted TEXT,      -- JSON of EncryptedData
                                           totp_secret_encrypted TEXT, -- JSON of EncryptedData
                                           tags TEXT NOT NULL DEFAULT '[]',
                                           color_name TEXT,
                                           color_hex TEXT,
                                           is_favorite INTEGER NOT NULL DEFAULT 0,
                                           trash_status TEXT NOT NULL DEFAULT 'Active',
                                           created_at INTEGER NOT NULL,
                                           updated_at INTEGER NOT NULL
);