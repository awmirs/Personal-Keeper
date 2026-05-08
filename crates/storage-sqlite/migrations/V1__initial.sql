CREATE TABLE IF NOT EXISTS notes (
                                     id TEXT PRIMARY KEY,
                                     title TEXT NOT NULL,
                                     content TEXT NOT NULL DEFAULT '',
                                     is_pinned INTEGER NOT NULL DEFAULT 0,
                                     tags TEXT NOT NULL DEFAULT '[]',          -- JSON array
                                     color_name TEXT,
                                     color_hex TEXT,
                                     is_favorite INTEGER NOT NULL DEFAULT 0,
                                     trash_status TEXT NOT NULL DEFAULT 'Active',
                                     created_at INTEGER NOT NULL,
                                     updated_at INTEGER NOT NULL
);