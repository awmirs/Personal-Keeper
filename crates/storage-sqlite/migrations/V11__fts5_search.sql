-- Migration V11: Full-text search indexes (SQLite FTS5)
--
-- Each searchable vault gets a contentless FTS5 index (content='') keyed
-- by the base table's implicit rowid. Contentless means the index stores
-- only the inverted tokens — no duplicated text — so the single .db file
-- stays compact. The base tables use `INSERT ... ON CONFLICT(id) DO
-- UPDATE`, which never deletes and re-inserts, so their implicit rowid is
-- stable and safe to key on. Do not switch those to `INSERT OR REPLACE`:
-- it would break both the versioning triggers (V10) and these indexes.
--
-- The `unicode61 remove_diacritics 2` tokenizer tokenizes Persian and
-- other non-Latin scripts correctly. FTS5 does not ship a Persian
-- stemmer, so query terms are prefix-matched in `fts5_query` instead.
--
-- Credential secrets (password_encrypted, notes_encrypted,
-- totp_secret_encrypted) are intentionally excluded from the index.

-- ------------------------------------------------------------------
-- notes: title, content
-- ------------------------------------------------------------------
CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
    title, content,
    content='',
    tokenize="unicode61 remove_diacritics 2"
);

INSERT INTO notes_fts(rowid, title, content)
SELECT rowid, title, content FROM notes;

CREATE TRIGGER IF NOT EXISTS trg_notes_fts_ai AFTER INSERT ON notes BEGIN
    INSERT INTO notes_fts(rowid, title, content) VALUES (new.rowid, new.title, new.content);
END;
CREATE TRIGGER IF NOT EXISTS trg_notes_fts_ad AFTER DELETE ON notes BEGIN
    INSERT INTO notes_fts(notes_fts, rowid, title, content)
        VALUES ('delete', old.rowid, old.title, old.content);
END;
CREATE TRIGGER IF NOT EXISTS trg_notes_fts_au AFTER UPDATE OF title, content ON notes BEGIN
    INSERT INTO notes_fts(notes_fts, rowid, title, content)
        VALUES ('delete', old.rowid, old.title, old.content);
    INSERT INTO notes_fts(rowid, title, content) VALUES (new.rowid, new.title, new.content);
END;

-- ------------------------------------------------------------------
-- clipboard_items: content
-- ------------------------------------------------------------------
CREATE VIRTUAL TABLE IF NOT EXISTS clipboard_fts USING fts5(
    content,
    content='',
    tokenize="unicode61 remove_diacritics 2"
);

INSERT INTO clipboard_fts(rowid, content)
SELECT rowid, content FROM clipboard_items;

CREATE TRIGGER IF NOT EXISTS trg_clipboard_fts_ai AFTER INSERT ON clipboard_items BEGIN
    INSERT INTO clipboard_fts(rowid, content) VALUES (new.rowid, new.content);
END;
CREATE TRIGGER IF NOT EXISTS trg_clipboard_fts_ad AFTER DELETE ON clipboard_items BEGIN
    INSERT INTO clipboard_fts(clipboard_fts, rowid, content)
        VALUES ('delete', old.rowid, old.content);
END;
CREATE TRIGGER IF NOT EXISTS trg_clipboard_fts_au AFTER UPDATE OF content ON clipboard_items BEGIN
    INSERT INTO clipboard_fts(clipboard_fts, rowid, content)
        VALUES ('delete', old.rowid, old.content);
    INSERT INTO clipboard_fts(rowid, content) VALUES (new.rowid, new.content);
END;

-- ------------------------------------------------------------------
-- todos: title, description
-- ------------------------------------------------------------------
CREATE VIRTUAL TABLE IF NOT EXISTS todos_fts USING fts5(
    title, description,
    content='',
    tokenize="unicode61 remove_diacritics 2"
);

INSERT INTO todos_fts(rowid, title, description)
SELECT rowid, title, description FROM todos;

CREATE TRIGGER IF NOT EXISTS trg_todos_fts_ai AFTER INSERT ON todos BEGIN
    INSERT INTO todos_fts(rowid, title, description)
        VALUES (new.rowid, new.title, new.description);
END;
CREATE TRIGGER IF NOT EXISTS trg_todos_fts_ad AFTER DELETE ON todos BEGIN
    INSERT INTO todos_fts(todos_fts, rowid, title, description)
        VALUES ('delete', old.rowid, old.title, old.description);
END;
CREATE TRIGGER IF NOT EXISTS trg_todos_fts_au AFTER UPDATE OF title, description ON todos BEGIN
    INSERT INTO todos_fts(todos_fts, rowid, title, description)
        VALUES ('delete', old.rowid, old.title, old.description);
    INSERT INTO todos_fts(rowid, title, description)
        VALUES (new.rowid, new.title, new.description);
END;

-- ------------------------------------------------------------------
-- bookmarks: url, title, description
-- ------------------------------------------------------------------
CREATE VIRTUAL TABLE IF NOT EXISTS bookmarks_fts USING fts5(
    url, title, description,
    content='',
    tokenize="unicode61 remove_diacritics 2"
);

INSERT INTO bookmarks_fts(rowid, url, title, description)
SELECT rowid, url, title, description FROM bookmarks;

CREATE TRIGGER IF NOT EXISTS trg_bookmarks_fts_ai AFTER INSERT ON bookmarks BEGIN
    INSERT INTO bookmarks_fts(rowid, url, title, description)
        VALUES (new.rowid, new.url, new.title, new.description);
END;
CREATE TRIGGER IF NOT EXISTS trg_bookmarks_fts_ad AFTER DELETE ON bookmarks BEGIN
    INSERT INTO bookmarks_fts(bookmarks_fts, rowid, url, title, description)
        VALUES ('delete', old.rowid, old.url, old.title, old.description);
END;
CREATE TRIGGER IF NOT EXISTS trg_bookmarks_fts_au AFTER UPDATE OF url, title, description ON bookmarks BEGIN
    INSERT INTO bookmarks_fts(bookmarks_fts, rowid, url, title, description)
        VALUES ('delete', old.rowid, old.url, old.title, old.description);
    INSERT INTO bookmarks_fts(rowid, url, title, description)
        VALUES (new.rowid, new.url, new.title, new.description);
END;

-- ------------------------------------------------------------------
-- contacts: name, phones, emails, addresses, notes
-- ------------------------------------------------------------------
CREATE VIRTUAL TABLE IF NOT EXISTS contacts_fts USING fts5(
    name, phones, emails, addresses, notes,
    content='',
    tokenize="unicode61 remove_diacritics 2"
);

INSERT INTO contacts_fts(rowid, name, phones, emails, addresses, notes)
SELECT rowid, name, phones, emails, addresses, notes FROM contacts;

CREATE TRIGGER IF NOT EXISTS trg_contacts_fts_ai AFTER INSERT ON contacts BEGIN
    INSERT INTO contacts_fts(rowid, name, phones, emails, addresses, notes)
        VALUES (new.rowid, new.name, new.phones, new.emails, new.addresses, new.notes);
END;
CREATE TRIGGER IF NOT EXISTS trg_contacts_fts_ad AFTER DELETE ON contacts BEGIN
    INSERT INTO contacts_fts(contacts_fts, rowid, name, phones, emails, addresses, notes)
        VALUES ('delete', old.rowid, old.name, old.phones, old.emails, old.addresses, old.notes);
END;
CREATE TRIGGER IF NOT EXISTS trg_contacts_fts_au AFTER UPDATE OF name, phones, emails, addresses, notes ON contacts BEGIN
    INSERT INTO contacts_fts(contacts_fts, rowid, name, phones, emails, addresses, notes)
        VALUES ('delete', old.rowid, old.name, old.phones, old.emails, old.addresses, old.notes);
    INSERT INTO contacts_fts(rowid, name, phones, emails, addresses, notes)
        VALUES (new.rowid, new.name, new.phones, new.emails, new.addresses, new.notes);
END;

-- ------------------------------------------------------------------
-- credentials: website, url, username — never any secret column
-- ------------------------------------------------------------------
CREATE VIRTUAL TABLE IF NOT EXISTS credentials_fts USING fts5(
    website, url, username,
    content='',
    tokenize="unicode61 remove_diacritics 2"
);

INSERT INTO credentials_fts(rowid, website, url, username)
SELECT rowid, website, url, username FROM credentials;

CREATE TRIGGER IF NOT EXISTS trg_credentials_fts_ai AFTER INSERT ON credentials BEGIN
    INSERT INTO credentials_fts(rowid, website, url, username)
        VALUES (new.rowid, new.website, new.url, new.username);
END;
CREATE TRIGGER IF NOT EXISTS trg_credentials_fts_ad AFTER DELETE ON credentials BEGIN
    INSERT INTO credentials_fts(credentials_fts, rowid, website, url, username)
        VALUES ('delete', old.rowid, old.website, old.url, old.username);
END;
CREATE TRIGGER IF NOT EXISTS trg_credentials_fts_au AFTER UPDATE OF website, url, username ON credentials BEGIN
    INSERT INTO credentials_fts(credentials_fts, rowid, website, url, username)
        VALUES ('delete', old.rowid, old.website, old.url, old.username);
    INSERT INTO credentials_fts(rowid, website, url, username)
        VALUES (new.rowid, new.website, new.url, new.username);
END;
