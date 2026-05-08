mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

pub fn run_migrations(pool: &crate::pool::DbPool) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = pool.get()?;
    // refinery needs a mutable connection reference; r2d2 provides a DerefMut to rusqlite::Connection
    embedded::migrations::runner().run(&mut *conn)?;
    Ok(())
}