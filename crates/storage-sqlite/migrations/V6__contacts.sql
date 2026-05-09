CREATE TABLE IF NOT EXISTS contacts (
                                        id TEXT PRIMARY KEY,
                                        name TEXT NOT NULL,
                                        phones TEXT NOT NULL DEFAULT '[]',   -- JSON array of strings
                                        emails TEXT NOT NULL DEFAULT '[]',   -- JSON array of strings
                                        addresses TEXT NOT NULL DEFAULT '[]', -- JSON array of strings
                                        notes TEXT NOT NULL DEFAULT '',
                                        tags TEXT NOT NULL DEFAULT '[]',
                                        color_name TEXT,
                                        color_hex TEXT,
                                        is_favorite INTEGER NOT NULL DEFAULT 0,
                                        trash_status TEXT NOT NULL DEFAULT 'Active',
                                        created_at INTEGER NOT NULL,
                                        updated_at INTEGER NOT NULL
);