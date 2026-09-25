-- Migration V9: Multi-Tenancy & Data Isolation

-- 1. Ensure fallback administrative user exists for legacy orphaned rows
INSERT OR IGNORE INTO users (id, username, password_hash, created_at, updated_at)
VALUES ('00000000-0000-0000-0000-000000000000', 'admin', '', 0, 0);

-- 2. Add user_id columns with NULL default (required by SQLite when foreign_keys enabled)
ALTER TABLE notes ADD COLUMN user_id TEXT DEFAULT NULL REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE clipboard_items ADD COLUMN user_id TEXT DEFAULT NULL REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE todos ADD COLUMN user_id TEXT DEFAULT NULL REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE bookmarks ADD COLUMN user_id TEXT DEFAULT NULL REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE contacts ADD COLUMN user_id TEXT DEFAULT NULL REFERENCES users(id) ON DELETE CASCADE;
ALTER TABLE credentials ADD COLUMN user_id TEXT DEFAULT NULL REFERENCES users(id) ON DELETE CASCADE;

-- 3. Backfill orphaned rows to the earliest real user, or fallback admin
UPDATE notes SET user_id = COALESCE(
    (SELECT id FROM users WHERE id != '00000000-0000-0000-0000-000000000000' ORDER BY created_at ASC LIMIT 1),
    '00000000-0000-0000-0000-000000000000'
) WHERE user_id IS NULL;

UPDATE clipboard_items SET user_id = COALESCE(
    (SELECT id FROM users WHERE id != '00000000-0000-0000-0000-000000000000' ORDER BY created_at ASC LIMIT 1),
    '00000000-0000-0000-0000-000000000000'
) WHERE user_id IS NULL;

UPDATE todos SET user_id = COALESCE(
    (SELECT id FROM users WHERE id != '00000000-0000-0000-0000-000000000000' ORDER BY created_at ASC LIMIT 1),
    '00000000-0000-0000-0000-000000000000'
) WHERE user_id IS NULL;

UPDATE bookmarks SET user_id = COALESCE(
    (SELECT id FROM users WHERE id != '00000000-0000-0000-0000-000000000000' ORDER BY created_at ASC LIMIT 1),
    '00000000-0000-0000-0000-000000000000'
) WHERE user_id IS NULL;

UPDATE contacts SET user_id = COALESCE(
    (SELECT id FROM users WHERE id != '00000000-0000-0000-0000-000000000000' ORDER BY created_at ASC LIMIT 1),
    '00000000-0000-0000-0000-000000000000'
) WHERE user_id IS NULL;

UPDATE credentials SET user_id = COALESCE(
    (SELECT id FROM users WHERE id != '00000000-0000-0000-0000-000000000000' ORDER BY created_at ASC LIMIT 1),
    '00000000-0000-0000-0000-000000000000'
) WHERE user_id IS NULL;

-- 4. Recreate credentials_config with user_id as primary key
CREATE TABLE IF NOT EXISTS credentials_config_new (
    user_id TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    password_hash TEXT NOT NULL,
    salt BLOB NOT NULL
);

INSERT OR IGNORE INTO credentials_config_new (user_id, password_hash, salt)
SELECT
    COALESCE(
        (SELECT id FROM users WHERE id != '00000000-0000-0000-0000-000000000000' ORDER BY created_at ASC LIMIT 1),
        '00000000-0000-0000-0000-000000000000'
    ),
    password_hash,
    salt
FROM credentials_config
WHERE id = 'master';

DROP TABLE credentials_config;
ALTER TABLE credentials_config_new RENAME TO credentials_config;

-- 5. Create multi-tenant composite ordering indexes
CREATE INDEX IF NOT EXISTS idx_notes_user_pos ON notes(user_id, position ASC);
CREATE INDEX IF NOT EXISTS idx_clipboard_user_pos ON clipboard_items(user_id, position ASC);
CREATE INDEX IF NOT EXISTS idx_todos_user_pos ON todos(user_id, position ASC);
CREATE INDEX IF NOT EXISTS idx_bookmarks_user_pos ON bookmarks(user_id, position ASC);
CREATE INDEX IF NOT EXISTS idx_contacts_user_pos ON contacts(user_id, position ASC);
CREATE INDEX IF NOT EXISTS idx_credentials_user_pos ON credentials(user_id, position ASC);
