ALTER TABLE notes ADD COLUMN position REAL NOT NULL DEFAULT 0.0;
ALTER TABLE clipboard_items ADD COLUMN position REAL NOT NULL DEFAULT 0.0;
ALTER TABLE todos ADD COLUMN position REAL NOT NULL DEFAULT 0.0;
ALTER TABLE bookmarks ADD COLUMN position REAL NOT NULL DEFAULT 0.0;
ALTER TABLE contacts ADD COLUMN position REAL NOT NULL DEFAULT 0.0;
ALTER TABLE credentials ADD COLUMN position REAL NOT NULL DEFAULT 0.0;

-- Backfill sequential positions so existing rows keep their current order
UPDATE notes SET position = (
                                SELECT COUNT(*) FROM notes AS t2
                                WHERE t2.created_at < notes.created_at
                                   OR (t2.created_at = notes.created_at AND t2.id < notes.id)
                            ) * 1.0;

UPDATE clipboard_items SET position = (
                                          SELECT COUNT(*) FROM clipboard_items AS t2
                                          WHERE t2.created_at < clipboard_items.created_at
                                             OR (t2.created_at = clipboard_items.created_at AND t2.id < clipboard_items.id)
                                      ) * 1.0;

UPDATE todos SET position = (
                                SELECT COUNT(*) FROM todos AS t2
                                WHERE t2.created_at < todos.created_at
                                   OR (t2.created_at = todos.created_at AND t2.id < todos.id)
                            ) * 1.0;

UPDATE bookmarks SET position = (
                                    SELECT COUNT(*) FROM bookmarks AS t2
                                    WHERE t2.created_at < bookmarks.created_at
                                       OR (t2.created_at = bookmarks.created_at AND t2.id < bookmarks.id)
                                ) * 1.0;

UPDATE contacts SET position = (
                                   SELECT COUNT(*) FROM contacts AS t2
                                   WHERE t2.created_at < contacts.created_at
                                      OR (t2.created_at = contacts.created_at AND t2.id < contacts.id)
                               ) * 1.0;

UPDATE credentials SET position = (
                                      SELECT COUNT(*) FROM credentials AS t2
                                      WHERE t2.created_at < credentials.created_at
                                         OR (t2.created_at = credentials.created_at AND t2.id < credentials.id)
                                  ) * 1.0;