use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub type DbPool = Pool<SqliteConnectionManager>;

pub fn create_pool(database_url: &str) -> Result<DbPool, r2d2::Error> {
    let manager = SqliteConnectionManager::file(database_url).with_init(|conn| {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;
             PRAGMA cache_size = -20000;",
        )
    });
    Pool::builder()
        .max_size(8)
        .connection_timeout(std::time::Duration::from_secs(5))
        .build(manager)
}