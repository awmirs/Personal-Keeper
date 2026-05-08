CREATE TABLE IF NOT EXISTS todos (
                                     id TEXT PRIMARY KEY,
                                     title TEXT NOT NULL,
                                     description TEXT NOT NULL DEFAULT '',
                                     completed INTEGER NOT NULL DEFAULT 0,
                                     due_date INTEGER,
                                     tags TEXT NOT NULL DEFAULT '[]',
                                     color_name TEXT,
                                     color_hex TEXT,
                                     is_favorite INTEGER NOT NULL DEFAULT 0,
                                     trash_status TEXT NOT NULL DEFAULT 'Active',
                                     created_at INTEGER NOT NULL,
                                     updated_at INTEGER NOT NULL
);