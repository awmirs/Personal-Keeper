-- Migration V12: Atomic position counter + batched reorder support
--
-- Adds a `position_counters` table that replaces the previous
-- read-MAX-then-write pattern in `get_next_position`. The counter row is
-- bumped atomically inside a single SQL statement (INSERT … ON CONFLICT
-- DO UPDATE … RETURNING), which closes the race that previously allowed
-- two concurrent creates to receive the same position.
--
-- Each row is keyed by (user_id, entity). The seed value is the base
-- table's current MAX(position) for that user, so the first call after
-- this migration returns the same value the old code would have
-- produced (MAX + 1000). Subsequent calls bump by 1000 inside the same
-- statement.
--
-- The counter table is not itself versioned or history-tracked: it is a
-- monotonic bookkeeping device, not user data.

CREATE TABLE IF NOT EXISTS position_counters (
    user_id       TEXT NOT NULL,
    entity        TEXT NOT NULL,
    next_position REAL NOT NULL,
    PRIMARY KEY (user_id, entity)
);

-- Seed from existing rows. `WHERE user_id IS NOT NULL` skips the
-- fallback-admin rows V9 may have left behind before the FK was
-- established; those get lazily seeded on first use of the counter.
INSERT INTO position_counters (user_id, entity, next_position)
SELECT user_id, 'notes', COALESCE(MAX(position), 0.0)
FROM notes
WHERE user_id IS NOT NULL
GROUP BY user_id
ON CONFLICT(user_id, entity) DO NOTHING;

INSERT INTO position_counters (user_id, entity, next_position)
SELECT user_id, 'clipboard_items', COALESCE(MAX(position), 0.0)
FROM clipboard_items
WHERE user_id IS NOT NULL
GROUP BY user_id
ON CONFLICT(user_id, entity) DO NOTHING;

INSERT INTO position_counters (user_id, entity, next_position)
SELECT user_id, 'todos', COALESCE(MAX(position), 0.0)
FROM todos
WHERE user_id IS NOT NULL
GROUP BY user_id
ON CONFLICT(user_id, entity) DO NOTHING;

INSERT INTO position_counters (user_id, entity, next_position)
SELECT user_id, 'bookmarks', COALESCE(MAX(position), 0.0)
FROM bookmarks
WHERE user_id IS NOT NULL
GROUP BY user_id
ON CONFLICT(user_id, entity) DO NOTHING;

INSERT INTO position_counters (user_id, entity, next_position)
SELECT user_id, 'contacts', COALESCE(MAX(position), 0.0)
FROM contacts
WHERE user_id IS NOT NULL
GROUP BY user_id
ON CONFLICT(user_id, entity) DO NOTHING;

INSERT INTO position_counters (user_id, entity, next_position)
SELECT user_id, 'credentials', COALESCE(MAX(position), 0.0)
FROM credentials
WHERE user_id IS NOT NULL
GROUP BY user_id
ON CONFLICT(user_id, entity) DO NOTHING;
