CREATE TABLE IF NOT EXISTS clipboard_items (
                                               id TEXT PRIMARY KEY,
                                               content TEXT NOT NULL,
                                               persist_to_disk INTEGER NOT NULL DEFAULT 1,
                                               tags TEXT NOT NULL DEFAULT '[]',
                                               color_name TEXT,
                                               color_hex TEXT,
                                               is_favorite INTEGER NOT NULL DEFAULT 0,
                                               trash_status TEXT NOT NULL DEFAULT 'Active',
                                               created_at INTEGER NOT NULL,
                                               updated_at INTEGER NOT NULL
);