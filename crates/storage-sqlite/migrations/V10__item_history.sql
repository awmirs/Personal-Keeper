-- Migration V10: Item History & Versioning
--
-- Adds a complete, automatic version history for every vault item.
-- SQLite triggers snapshot each row on INSERT (created), UPDATE
-- (updated) and DELETE (deleted) into `item_versions`; every create,
-- edit, favorite/pin toggle, trash/restore, reorder and hard delete is
-- recorded. Existing rows are backfilled with an initial baseline
-- version. Binary columns (bookmark favicon/thumbnail) are not
-- snapshotted; restoring keeps the current artwork.

CREATE TABLE IF NOT EXISTS item_versions (
    id TEXT PRIMARY KEY,
    item_type TEXT NOT NULL CHECK (item_type IN ('note', 'clipboard', 'todo', 'bookmark', 'contact', 'credential')),
    item_id TEXT NOT NULL,
    user_id TEXT NOT NULL DEFAULT '',
    version INTEGER NOT NULL,
    operation TEXT NOT NULL CHECK (operation IN ('created', 'updated', 'deleted', 'restored')),
    data TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    UNIQUE (item_type, item_id, version)
);

CREATE INDEX IF NOT EXISTS idx_item_versions_item ON item_versions (item_type, item_id, version DESC);
CREATE INDEX IF NOT EXISTS idx_item_versions_user_time ON item_versions (user_id, created_at DESC);

-- ------------------------------------------------------------------
-- note (notes)
-- ------------------------------------------------------------------

-- Baseline version for rows that already exist.
INSERT INTO item_versions (id, item_type, item_id, user_id, version, operation, data, created_at)
SELECT
    lower(hex(randomblob(16))),
    'note',
    id,
    COALESCE(user_id, ''),
    1,
    'created',
    json_object(
        'id', id,
            'title', title,
            'content', content,
            'is_pinned', is_pinned,
            'tags', CASE WHEN json_valid(tags) THEN json(tags) ELSE tags END,
            'color_name', color_name,
            'color_hex', color_hex,
            'is_favorite', is_favorite,
            'trash_status', trash_status,
            'position', position,
            'created_at', created_at,
            'updated_at', updated_at
    ),
    created_at
FROM notes;

CREATE TRIGGER IF NOT EXISTS trg_notes_history_insert
AFTER INSERT ON notes
BEGIN
    INSERT INTO item_versions (id, item_type, item_id, user_id, version, operation, data, created_at)
    VALUES (
        lower(hex(randomblob(16))),
        'note',
        NEW.id,
        COALESCE(NEW.user_id, ''),
        COALESCE((SELECT MAX(version) FROM item_versions WHERE item_type = 'note' AND item_id = NEW.id), 0) + 1,
        'created',
        json_object(
            'id', NEW.id,
            'title', NEW.title,
            'content', NEW.content,
            'is_pinned', NEW.is_pinned,
            'tags', CASE WHEN json_valid(NEW.tags) THEN json(NEW.tags) ELSE NEW.tags END,
            'color_name', NEW.color_name,
            'color_hex', NEW.color_hex,
            'is_favorite', NEW.is_favorite,
            'trash_status', NEW.trash_status,
            'position', NEW.position,
            'created_at', NEW.created_at,
            'updated_at', NEW.updated_at
        ),
        NEW.updated_at
    );
END;

CREATE TRIGGER IF NOT EXISTS trg_notes_history_update
AFTER UPDATE OF title, content, is_pinned, tags, color_name, color_hex, is_favorite, trash_status, position ON notes
WHEN OLD.title IS NOT NEW.title
   OR OLD.content IS NOT NEW.content
   OR OLD.is_pinned IS NOT NEW.is_pinned
   OR OLD.tags IS NOT NEW.tags
   OR OLD.color_name IS NOT NEW.color_name
   OR OLD.color_hex IS NOT NEW.color_hex
   OR OLD.is_favorite IS NOT NEW.is_favorite
   OR OLD.trash_status IS NOT NEW.trash_status
   OR OLD.position IS NOT NEW.position
BEGIN
    INSERT INTO item_versions (id, item_type, item_id, user_id, version, operation, data, created_at)
    VALUES (
        lower(hex(randomblob(16))),
        'note',
        NEW.id,
        COALESCE(NEW.user_id, ''),
        COALESCE((SELECT MAX(version) FROM item_versions WHERE item_type = 'note' AND item_id = NEW.id), 0) + 1,
        'updated',
        json_object(
            'id', NEW.id,
            'title', NEW.title,
            'content', NEW.content,
            'is_pinned', NEW.is_pinned,
            'tags', CASE WHEN json_valid(NEW.tags) THEN json(NEW.tags) ELSE NEW.tags END,
            'color_name', NEW.color_name,
            'color_hex', NEW.color_hex,
            'is_favorite', NEW.is_favorite,
            'trash_status', NEW.trash_status,
            'position', NEW.position,
            'created_at', NEW.created_at,
            'updated_at', NEW.updated_at
        ),
        NEW.updated_at
    );
END;

CREATE TRIGGER IF NOT EXISTS trg_notes_history_delete
AFTER DELETE ON notes
BEGIN
    INSERT INTO item_versions (id, item_type, item_id, user_id, version, operation, data, created_at)
    VALUES (
        lower(hex(randomblob(16))),
        'note',
        OLD.id,
        COALESCE(OLD.user_id, ''),
        COALESCE((SELECT MAX(version) FROM item_versions WHERE item_type = 'note' AND item_id = OLD.id), 0) + 1,
        'deleted',
        json_object(
            'id', OLD.id,
            'title', OLD.title,
            'content', OLD.content,
            'is_pinned', OLD.is_pinned,
            'tags', CASE WHEN json_valid(OLD.tags) THEN json(OLD.tags) ELSE OLD.tags END,
            'color_name', OLD.color_name,
            'color_hex', OLD.color_hex,
            'is_favorite', OLD.is_favorite,
            'trash_status', OLD.trash_status,
            'position', OLD.position,
            'created_at', OLD.created_at,
            'updated_at', OLD.updated_at
        ),
        CAST(strftime('%s', 'now') AS INTEGER)
    );
END;

-- ------------------------------------------------------------------
-- clipboard (clipboard_items)
-- ------------------------------------------------------------------

-- Baseline version for rows that already exist.
INSERT INTO item_versions (id, item_type, item_id, user_id, version, operation, data, created_at)
SELECT
    lower(hex(randomblob(16))),
    'clipboard',
    id,
    COALESCE(user_id, ''),
    1,
    'created',
    json_object(
        'id', id,
            'content', content,
            'persist_to_disk', persist_to_disk,
            'tags', CASE WHEN json_valid(tags) THEN json(tags) ELSE tags END,
            'color_name', color_name,
            'color_hex', color_hex,
            'is_favorite', is_favorite,
            'trash_status', trash_status,
            'position', position,
            'created_at', created_at,
            'updated_at', updated_at
    ),
    created_at
FROM clipboard_items;

CREATE TRIGGER IF NOT EXISTS trg_clipboard_items_history_insert
AFTER INSERT ON clipboard_items
BEGIN
    INSERT INTO item_versions (id, item_type, item_id, user_id, version, operation, data, created_at)
    VALUES (
        lower(hex(randomblob(16))),
        'clipboard',
        NEW.id,
        COALESCE(NEW.user_id, ''),
        COALESCE((SELECT MAX(version) FROM item_versions WHERE item_type = 'clipboard' AND item_id = NEW.id), 0) + 1,
        'created',
        json_object(
            'id', NEW.id,
            'content', NEW.content,
            'persist_to_disk', NEW.persist_to_disk,
            'tags', CASE WHEN json_valid(NEW.tags) THEN json(NEW.tags) ELSE NEW.tags END,
            'color_name', NEW.color_name,
            'color_hex', NEW.color_hex,
            'is_favorite', NEW.is_favorite,
            'trash_status', NEW.trash_status,
            'position', NEW.position,
            'created_at', NEW.created_at,
            'updated_at', NEW.updated_at
        ),
        NEW.updated_at
    );
END;

CREATE TRIGGER IF NOT EXISTS trg_clipboard_items_history_update
AFTER UPDATE OF content, persist_to_disk, tags, color_name, color_hex, is_favorite, trash_status, position ON clipboard_items
WHEN OLD.content IS NOT NEW.content
   OR OLD.persist_to_disk IS NOT NEW.persist_to_disk
   OR OLD.tags IS NOT NEW.tags
   OR OLD.color_name IS NOT NEW.color_name
   OR OLD.color_hex IS NOT NEW.color_hex
   OR OLD.is_favorite IS NOT NEW.is_favorite
   OR OLD.trash_status IS NOT NEW.trash_status
   OR OLD.position IS NOT NEW.position
BEGIN
    INSERT INTO item_versions (id, item_type, item_id, user_id, version, operation, data, created_at)
    VALUES (
        lower(hex(randomblob(16))),
        'clipboard',
        NEW.id,
        COALESCE(NEW.user_id, ''),
        COALESCE((SELECT MAX(version) FROM item_versions WHERE item_type = 'clipboard' AND item_id = NEW.id), 0) + 1,
        'updated',
        json_object(
            'id', NEW.id,
            'content', NEW.content,
            'persist_to_disk', NEW.persist_to_disk,
            'tags', CASE WHEN json_valid(NEW.tags) THEN json(NEW.tags) ELSE NEW.tags END,
            'color_name', NEW.color_name,
            'color_hex', NEW.color_hex,
            'is_favorite', NEW.is_favorite,
            'trash_status', NEW.trash_status,
            'position', NEW.position,
            'created_at', NEW.created_at,
            'updated_at', NEW.updated_at
        ),
        NEW.updated_at
    );
END;

CREATE TRIGGER IF NOT EXISTS trg_clipboard_items_history_delete
AFTER DELETE ON clipboard_items
BEGIN
    INSERT INTO item_versions (id, item_type, item_id, user_id, version, operation, data, created_at)
    VALUES (
        lower(hex(randomblob(16))),
        'clipboard',
        OLD.id,
        COALESCE(OLD.user_id, ''),
        COALESCE((SELECT MAX(version) FROM item_versions WHERE item_type = 'clipboard' AND item_id = OLD.id), 0) + 1,
        'deleted',
        json_object(
            'id', OLD.id,
            'content', OLD.content,
            'persist_to_disk', OLD.persist_to_disk,
            'tags', CASE WHEN json_valid(OLD.tags) THEN json(OLD.tags) ELSE OLD.tags END,
            'color_name', OLD.color_name,
            'color_hex', OLD.color_hex,
            'is_favorite', OLD.is_favorite,
            'trash_status', OLD.trash_status,
            'position', OLD.position,
            'created_at', OLD.created_at,
            'updated_at', OLD.updated_at
        ),
        CAST(strftime('%s', 'now') AS INTEGER)
    );
END;

-- ------------------------------------------------------------------
-- todo (todos)
-- ------------------------------------------------------------------

-- Baseline version for rows that already exist.
INSERT INTO item_versions (id, item_type, item_id, user_id, version, operation, data, created_at)
SELECT
    lower(hex(randomblob(16))),
    'todo',
    id,
    COALESCE(user_id, ''),
    1,
    'created',
    json_object(
        'id', id,
            'title', title,
            'description', description,
            'completed', completed,
            'due_date', due_date,
            'tags', CASE WHEN json_valid(tags) THEN json(tags) ELSE tags END,
            'color_name', color_name,
            'color_hex', color_hex,
            'is_favorite', is_favorite,
            'trash_status', trash_status,
            'position', position,
            'created_at', created_at,
            'updated_at', updated_at
    ),
    created_at
FROM todos;

CREATE TRIGGER IF NOT EXISTS trg_todos_history_insert
AFTER INSERT ON todos
BEGIN
    INSERT INTO item_versions (id, item_type, item_id, user_id, version, operation, data, created_at)
    VALUES (
        lower(hex(randomblob(16))),
        'todo',
        NEW.id,
        COALESCE(NEW.user_id, ''),
        COALESCE((SELECT MAX(version) FROM item_versions WHERE item_type = 'todo' AND item_id = NEW.id), 0) + 1,
        'created',
        json_object(
            'id', NEW.id,
            'title', NEW.title,
            'description', NEW.description,
            'completed', NEW.completed,
            'due_date', NEW.due_date,
            'tags', CASE WHEN json_valid(NEW.tags) THEN json(NEW.tags) ELSE NEW.tags END,
            'color_name', NEW.color_name,
            'color_hex', NEW.color_hex,
            'is_favorite', NEW.is_favorite,
            'trash_status', NEW.trash_status,
            'position', NEW.position,
            'created_at', NEW.created_at,
            'updated_at', NEW.updated_at
        ),
        NEW.updated_at
    );
END;

CREATE TRIGGER IF NOT EXISTS trg_todos_history_update
AFTER UPDATE OF title, description, completed, due_date, tags, color_name, color_hex, is_favorite, trash_status, position ON todos
WHEN OLD.title IS NOT NEW.title
   OR OLD.description IS NOT NEW.description
   OR OLD.completed IS NOT NEW.completed
   OR OLD.due_date IS NOT NEW.due_date
   OR OLD.tags IS NOT NEW.tags
   OR OLD.color_name IS NOT NEW.color_name
   OR OLD.color_hex IS NOT NEW.color_hex
   OR OLD.is_favorite IS NOT NEW.is_favorite
   OR OLD.trash_status IS NOT NEW.trash_status
   OR OLD.position IS NOT NEW.position
BEGIN
    INSERT INTO item_versions (id, item_type, item_id, user_id, version, operation, data, created_at)
    VALUES (
        lower(hex(randomblob(16))),
        'todo',
        NEW.id,
        COALESCE(NEW.user_id, ''),
        COALESCE((SELECT MAX(version) FROM item_versions WHERE item_type = 'todo' AND item_id = NEW.id), 0) + 1,
        'updated',
        json_object(
            'id', NEW.id,
            'title', NEW.title,
            'description', NEW.description,
            'completed', NEW.completed,
            'due_date', NEW.due_date,
            'tags', CASE WHEN json_valid(NEW.tags) THEN json(NEW.tags) ELSE NEW.tags END,
            'color_name', NEW.color_name,
            'color_hex', NEW.color_hex,
            'is_favorite', NEW.is_favorite,
            'trash_status', NEW.trash_status,
            'position', NEW.position,
            'created_at', NEW.created_at,
            'updated_at', NEW.updated_at
        ),
        NEW.updated_at
    );
END;

CREATE TRIGGER IF NOT EXISTS trg_todos_history_delete
AFTER DELETE ON todos
BEGIN
    INSERT INTO item_versions (id, item_type, item_id, user_id, version, operation, data, created_at)
    VALUES (
        lower(hex(randomblob(16))),
        'todo',
        OLD.id,
        COALESCE(OLD.user_id, ''),
        COALESCE((SELECT MAX(version) FROM item_versions WHERE item_type = 'todo' AND item_id = OLD.id), 0) + 1,
        'deleted',
        json_object(
            'id', OLD.id,
            'title', OLD.title,
            'description', OLD.description,
            'completed', OLD.completed,
            'due_date', OLD.due_date,
            'tags', CASE WHEN json_valid(OLD.tags) THEN json(OLD.tags) ELSE OLD.tags END,
            'color_name', OLD.color_name,
            'color_hex', OLD.color_hex,
            'is_favorite', OLD.is_favorite,
            'trash_status', OLD.trash_status,
            'position', OLD.position,
            'created_at', OLD.created_at,
            'updated_at', OLD.updated_at
        ),
        CAST(strftime('%s', 'now') AS INTEGER)
    );
END;

-- ------------------------------------------------------------------
-- bookmark (bookmarks)
-- ------------------------------------------------------------------

-- Baseline version for rows that already exist.
INSERT INTO item_versions (id, item_type, item_id, user_id, version, operation, data, created_at)
SELECT
    lower(hex(randomblob(16))),
    'bookmark',
    id,
    COALESCE(user_id, ''),
    1,
    'created',
    json_object(
        'id', id,
            'url', url,
            'title', title,
            'description', description,
            'tags', CASE WHEN json_valid(tags) THEN json(tags) ELSE tags END,
            'color_name', color_name,
            'color_hex', color_hex,
            'is_favorite', is_favorite,
            'trash_status', trash_status,
            'position', position,
            'created_at', created_at,
            'updated_at', updated_at
    ),
    created_at
FROM bookmarks;

CREATE TRIGGER IF NOT EXISTS trg_bookmarks_history_insert
AFTER INSERT ON bookmarks
BEGIN
    INSERT INTO item_versions (id, item_type, item_id, user_id, version, operation, data, created_at)
    VALUES (
        lower(hex(randomblob(16))),
        'bookmark',
        NEW.id,
        COALESCE(NEW.user_id, ''),
        COALESCE((SELECT MAX(version) FROM item_versions WHERE item_type = 'bookmark' AND item_id = NEW.id), 0) + 1,
        'created',
        json_object(
            'id', NEW.id,
            'url', NEW.url,
            'title', NEW.title,
            'description', NEW.description,
            'tags', CASE WHEN json_valid(NEW.tags) THEN json(NEW.tags) ELSE NEW.tags END,
            'color_name', NEW.color_name,
            'color_hex', NEW.color_hex,
            'is_favorite', NEW.is_favorite,
            'trash_status', NEW.trash_status,
            'position', NEW.position,
            'created_at', NEW.created_at,
            'updated_at', NEW.updated_at
        ),
        NEW.updated_at
    );
END;

CREATE TRIGGER IF NOT EXISTS trg_bookmarks_history_update
AFTER UPDATE OF url, title, description, tags, color_name, color_hex, is_favorite, trash_status, position ON bookmarks
WHEN OLD.url IS NOT NEW.url
   OR OLD.title IS NOT NEW.title
   OR OLD.description IS NOT NEW.description
   OR OLD.tags IS NOT NEW.tags
   OR OLD.color_name IS NOT NEW.color_name
   OR OLD.color_hex IS NOT NEW.color_hex
   OR OLD.is_favorite IS NOT NEW.is_favorite
   OR OLD.trash_status IS NOT NEW.trash_status
   OR OLD.position IS NOT NEW.position
BEGIN
    INSERT INTO item_versions (id, item_type, item_id, user_id, version, operation, data, created_at)
    VALUES (
        lower(hex(randomblob(16))),
        'bookmark',
        NEW.id,
        COALESCE(NEW.user_id, ''),
        COALESCE((SELECT MAX(version) FROM item_versions WHERE item_type = 'bookmark' AND item_id = NEW.id), 0) + 1,
        'updated',
        json_object(
            'id', NEW.id,
            'url', NEW.url,
            'title', NEW.title,
            'description', NEW.description,
            'tags', CASE WHEN json_valid(NEW.tags) THEN json(NEW.tags) ELSE NEW.tags END,
            'color_name', NEW.color_name,
            'color_hex', NEW.color_hex,
            'is_favorite', NEW.is_favorite,
            'trash_status', NEW.trash_status,
            'position', NEW.position,
            'created_at', NEW.created_at,
            'updated_at', NEW.updated_at
        ),
        NEW.updated_at
    );
END;

CREATE TRIGGER IF NOT EXISTS trg_bookmarks_history_delete
AFTER DELETE ON bookmarks
BEGIN
    INSERT INTO item_versions (id, item_type, item_id, user_id, version, operation, data, created_at)
    VALUES (
        lower(hex(randomblob(16))),
        'bookmark',
        OLD.id,
        COALESCE(OLD.user_id, ''),
        COALESCE((SELECT MAX(version) FROM item_versions WHERE item_type = 'bookmark' AND item_id = OLD.id), 0) + 1,
        'deleted',
        json_object(
            'id', OLD.id,
            'url', OLD.url,
            'title', OLD.title,
            'description', OLD.description,
            'tags', CASE WHEN json_valid(OLD.tags) THEN json(OLD.tags) ELSE OLD.tags END,
            'color_name', OLD.color_name,
            'color_hex', OLD.color_hex,
            'is_favorite', OLD.is_favorite,
            'trash_status', OLD.trash_status,
            'position', OLD.position,
            'created_at', OLD.created_at,
            'updated_at', OLD.updated_at
        ),
        CAST(strftime('%s', 'now') AS INTEGER)
    );
END;

-- ------------------------------------------------------------------
-- contact (contacts)
-- ------------------------------------------------------------------

-- Baseline version for rows that already exist.
INSERT INTO item_versions (id, item_type, item_id, user_id, version, operation, data, created_at)
SELECT
    lower(hex(randomblob(16))),
    'contact',
    id,
    COALESCE(user_id, ''),
    1,
    'created',
    json_object(
        'id', id,
            'name', name,
            'phones', CASE WHEN json_valid(phones) THEN json(phones) ELSE phones END,
            'emails', CASE WHEN json_valid(emails) THEN json(emails) ELSE emails END,
            'addresses', CASE WHEN json_valid(addresses) THEN json(addresses) ELSE addresses END,
            'notes', notes,
            'tags', CASE WHEN json_valid(tags) THEN json(tags) ELSE tags END,
            'color_name', color_name,
            'color_hex', color_hex,
            'is_favorite', is_favorite,
            'trash_status', trash_status,
            'position', position,
            'created_at', created_at,
            'updated_at', updated_at
    ),
    created_at
FROM contacts;

CREATE TRIGGER IF NOT EXISTS trg_contacts_history_insert
AFTER INSERT ON contacts
BEGIN
    INSERT INTO item_versions (id, item_type, item_id, user_id, version, operation, data, created_at)
    VALUES (
        lower(hex(randomblob(16))),
        'contact',
        NEW.id,
        COALESCE(NEW.user_id, ''),
        COALESCE((SELECT MAX(version) FROM item_versions WHERE item_type = 'contact' AND item_id = NEW.id), 0) + 1,
        'created',
        json_object(
            'id', NEW.id,
            'name', NEW.name,
            'phones', CASE WHEN json_valid(NEW.phones) THEN json(NEW.phones) ELSE NEW.phones END,
            'emails', CASE WHEN json_valid(NEW.emails) THEN json(NEW.emails) ELSE NEW.emails END,
            'addresses', CASE WHEN json_valid(NEW.addresses) THEN json(NEW.addresses) ELSE NEW.addresses END,
            'notes', NEW.notes,
            'tags', CASE WHEN json_valid(NEW.tags) THEN json(NEW.tags) ELSE NEW.tags END,
            'color_name', NEW.color_name,
            'color_hex', NEW.color_hex,
            'is_favorite', NEW.is_favorite,
            'trash_status', NEW.trash_status,
            'position', NEW.position,
            'created_at', NEW.created_at,
            'updated_at', NEW.updated_at
        ),
        NEW.updated_at
    );
END;

CREATE TRIGGER IF NOT EXISTS trg_contacts_history_update
AFTER UPDATE OF name, phones, emails, addresses, notes, tags, color_name, color_hex, is_favorite, trash_status, position ON contacts
WHEN OLD.name IS NOT NEW.name
   OR OLD.phones IS NOT NEW.phones
   OR OLD.emails IS NOT NEW.emails
   OR OLD.addresses IS NOT NEW.addresses
   OR OLD.notes IS NOT NEW.notes
   OR OLD.tags IS NOT NEW.tags
   OR OLD.color_name IS NOT NEW.color_name
   OR OLD.color_hex IS NOT NEW.color_hex
   OR OLD.is_favorite IS NOT NEW.is_favorite
   OR OLD.trash_status IS NOT NEW.trash_status
   OR OLD.position IS NOT NEW.position
BEGIN
    INSERT INTO item_versions (id, item_type, item_id, user_id, version, operation, data, created_at)
    VALUES (
        lower(hex(randomblob(16))),
        'contact',
        NEW.id,
        COALESCE(NEW.user_id, ''),
        COALESCE((SELECT MAX(version) FROM item_versions WHERE item_type = 'contact' AND item_id = NEW.id), 0) + 1,
        'updated',
        json_object(
            'id', NEW.id,
            'name', NEW.name,
            'phones', CASE WHEN json_valid(NEW.phones) THEN json(NEW.phones) ELSE NEW.phones END,
            'emails', CASE WHEN json_valid(NEW.emails) THEN json(NEW.emails) ELSE NEW.emails END,
            'addresses', CASE WHEN json_valid(NEW.addresses) THEN json(NEW.addresses) ELSE NEW.addresses END,
            'notes', NEW.notes,
            'tags', CASE WHEN json_valid(NEW.tags) THEN json(NEW.tags) ELSE NEW.tags END,
            'color_name', NEW.color_name,
            'color_hex', NEW.color_hex,
            'is_favorite', NEW.is_favorite,
            'trash_status', NEW.trash_status,
            'position', NEW.position,
            'created_at', NEW.created_at,
            'updated_at', NEW.updated_at
        ),
        NEW.updated_at
    );
END;

CREATE TRIGGER IF NOT EXISTS trg_contacts_history_delete
AFTER DELETE ON contacts
BEGIN
    INSERT INTO item_versions (id, item_type, item_id, user_id, version, operation, data, created_at)
    VALUES (
        lower(hex(randomblob(16))),
        'contact',
        OLD.id,
        COALESCE(OLD.user_id, ''),
        COALESCE((SELECT MAX(version) FROM item_versions WHERE item_type = 'contact' AND item_id = OLD.id), 0) + 1,
        'deleted',
        json_object(
            'id', OLD.id,
            'name', OLD.name,
            'phones', CASE WHEN json_valid(OLD.phones) THEN json(OLD.phones) ELSE OLD.phones END,
            'emails', CASE WHEN json_valid(OLD.emails) THEN json(OLD.emails) ELSE OLD.emails END,
            'addresses', CASE WHEN json_valid(OLD.addresses) THEN json(OLD.addresses) ELSE OLD.addresses END,
            'notes', OLD.notes,
            'tags', CASE WHEN json_valid(OLD.tags) THEN json(OLD.tags) ELSE OLD.tags END,
            'color_name', OLD.color_name,
            'color_hex', OLD.color_hex,
            'is_favorite', OLD.is_favorite,
            'trash_status', OLD.trash_status,
            'position', OLD.position,
            'created_at', OLD.created_at,
            'updated_at', OLD.updated_at
        ),
        CAST(strftime('%s', 'now') AS INTEGER)
    );
END;

-- ------------------------------------------------------------------
-- credential (credentials)
-- ------------------------------------------------------------------

-- Baseline version for rows that already exist.
INSERT INTO item_versions (id, item_type, item_id, user_id, version, operation, data, created_at)
SELECT
    lower(hex(randomblob(16))),
    'credential',
    id,
    COALESCE(user_id, ''),
    1,
    'created',
    json_object(
        'id', id,
            'website', website,
            'url', url,
            'username', username,
            'password_encrypted', password_encrypted,
            'notes_encrypted', notes_encrypted,
            'totp_secret_encrypted', totp_secret_encrypted,
            'tags', CASE WHEN json_valid(tags) THEN json(tags) ELSE tags END,
            'color_name', color_name,
            'color_hex', color_hex,
            'is_favorite', is_favorite,
            'trash_status', trash_status,
            'position', position,
            'created_at', created_at,
            'updated_at', updated_at
    ),
    created_at
FROM credentials;

CREATE TRIGGER IF NOT EXISTS trg_credentials_history_insert
AFTER INSERT ON credentials
BEGIN
    INSERT INTO item_versions (id, item_type, item_id, user_id, version, operation, data, created_at)
    VALUES (
        lower(hex(randomblob(16))),
        'credential',
        NEW.id,
        COALESCE(NEW.user_id, ''),
        COALESCE((SELECT MAX(version) FROM item_versions WHERE item_type = 'credential' AND item_id = NEW.id), 0) + 1,
        'created',
        json_object(
            'id', NEW.id,
            'website', NEW.website,
            'url', NEW.url,
            'username', NEW.username,
            'password_encrypted', NEW.password_encrypted,
            'notes_encrypted', NEW.notes_encrypted,
            'totp_secret_encrypted', NEW.totp_secret_encrypted,
            'tags', CASE WHEN json_valid(NEW.tags) THEN json(NEW.tags) ELSE NEW.tags END,
            'color_name', NEW.color_name,
            'color_hex', NEW.color_hex,
            'is_favorite', NEW.is_favorite,
            'trash_status', NEW.trash_status,
            'position', NEW.position,
            'created_at', NEW.created_at,
            'updated_at', NEW.updated_at
        ),
        NEW.updated_at
    );
END;

CREATE TRIGGER IF NOT EXISTS trg_credentials_history_update
AFTER UPDATE OF website, url, username, password_encrypted, notes_encrypted, totp_secret_encrypted, tags, color_name, color_hex, is_favorite, trash_status, position ON credentials
WHEN OLD.website IS NOT NEW.website
   OR OLD.url IS NOT NEW.url
   OR OLD.username IS NOT NEW.username
   OR OLD.password_encrypted IS NOT NEW.password_encrypted
   OR OLD.notes_encrypted IS NOT NEW.notes_encrypted
   OR OLD.totp_secret_encrypted IS NOT NEW.totp_secret_encrypted
   OR OLD.tags IS NOT NEW.tags
   OR OLD.color_name IS NOT NEW.color_name
   OR OLD.color_hex IS NOT NEW.color_hex
   OR OLD.is_favorite IS NOT NEW.is_favorite
   OR OLD.trash_status IS NOT NEW.trash_status
   OR OLD.position IS NOT NEW.position
BEGIN
    INSERT INTO item_versions (id, item_type, item_id, user_id, version, operation, data, created_at)
    VALUES (
        lower(hex(randomblob(16))),
        'credential',
        NEW.id,
        COALESCE(NEW.user_id, ''),
        COALESCE((SELECT MAX(version) FROM item_versions WHERE item_type = 'credential' AND item_id = NEW.id), 0) + 1,
        'updated',
        json_object(
            'id', NEW.id,
            'website', NEW.website,
            'url', NEW.url,
            'username', NEW.username,
            'password_encrypted', NEW.password_encrypted,
            'notes_encrypted', NEW.notes_encrypted,
            'totp_secret_encrypted', NEW.totp_secret_encrypted,
            'tags', CASE WHEN json_valid(NEW.tags) THEN json(NEW.tags) ELSE NEW.tags END,
            'color_name', NEW.color_name,
            'color_hex', NEW.color_hex,
            'is_favorite', NEW.is_favorite,
            'trash_status', NEW.trash_status,
            'position', NEW.position,
            'created_at', NEW.created_at,
            'updated_at', NEW.updated_at
        ),
        NEW.updated_at
    );
END;

CREATE TRIGGER IF NOT EXISTS trg_credentials_history_delete
AFTER DELETE ON credentials
BEGIN
    INSERT INTO item_versions (id, item_type, item_id, user_id, version, operation, data, created_at)
    VALUES (
        lower(hex(randomblob(16))),
        'credential',
        OLD.id,
        COALESCE(OLD.user_id, ''),
        COALESCE((SELECT MAX(version) FROM item_versions WHERE item_type = 'credential' AND item_id = OLD.id), 0) + 1,
        'deleted',
        json_object(
            'id', OLD.id,
            'website', OLD.website,
            'url', OLD.url,
            'username', OLD.username,
            'password_encrypted', OLD.password_encrypted,
            'notes_encrypted', OLD.notes_encrypted,
            'totp_secret_encrypted', OLD.totp_secret_encrypted,
            'tags', CASE WHEN json_valid(OLD.tags) THEN json(OLD.tags) ELSE OLD.tags END,
            'color_name', OLD.color_name,
            'color_hex', OLD.color_hex,
            'is_favorite', OLD.is_favorite,
            'trash_status', OLD.trash_status,
            'position', OLD.position,
            'created_at', OLD.created_at,
            'updated_at', OLD.updated_at
        ),
        CAST(strftime('%s', 'now') AS INTEGER)
    );
END;
