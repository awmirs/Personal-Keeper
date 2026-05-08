use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

pub type DbPool = Pool<SqliteConnectionManager>;

pub fn create_pool(database_url: &str) -> Result<DbPool, r2d2::Error> {
    let manager = SqliteConnectionManager::file(database_url);
    Pool::builder().max_size(4).build(manager)
}