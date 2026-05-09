CREATE TABLE IF NOT EXISTS bookmarks (
                                         id TEXT PRIMARY KEY,
                                         url TEXT NOT NULL,
                                         title TEXT NOT NULL DEFAULT '',
                                         description TEXT NOT NULL DEFAULT '',
                                         favicon BLOB,
                                         thumbnail BLOB,
                                         tags TEXT NOT NULL DEFAULT '[]',
                                         color_name TEXT,
                                         color_hex TEXT,
                                         is_favorite INTEGER NOT NULL DEFAULT 0,
                                         trash_status TEXT NOT NULL DEFAULT 'Active',
                                         created_at INTEGER NOT NULL,
                                         updated_at INTEGER NOT NULL
);