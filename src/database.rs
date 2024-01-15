use rusqlite::{Connection, Result};

use crate::runtime_err::RunTimeError;
pub fn get_conn() -> Result<Connection, RunTimeError> {
    let conn = Connection::open("data/tiktok.db").map_err(|e| RunTimeError::DatabaseError(e))?;
    conn.busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|e| RunTimeError::DatabaseError(e))?;
    let journal_mode: String = conn.query_row("PRAGMA journal_mode=WAL;", [], |row| row.get(0))?;
    let mmap_size: i64 = conn.query_row("PRAGMA mmap_size=30000000000;", [], |row| row.get(0))?;
    log::debug!("journal_mode: {} mmap_size: {}", journal_mode, mmap_size);
    conn.execute("PRAGMA cache_size=-64000;", [])?; // Set cache size to 64000 KB
    conn.execute("PRAGMA synchronous=NORMAL;", [])?; // Enable concurrent write
    conn.execute("PRAGMA temp_store=MEMORY;", [])?; // Use memory to store temporary data
    conn.execute("PRAGMA page_size=32768;", [])?; // Set page size to 32KB
    return Ok(conn);
}
pub fn create_databases() -> Result<(), RunTimeError> {
    let conn = get_conn()?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS `group` (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            title TEXT DEFAULT NULL,
            tags TEXT DEFAULT NULL,
            auto_publish INTEGER NOT NULL DEFAULT 1,
            publish_start_time TEXT DEFAULT '02:10',
            auto_train INTEGER NOT NULL DEFAULT 1,
            publish_type INTEGER NOT NULL DEFAULT 1,
            product_link TEXT DEFAULT NULL,
            train_start_time TEXT DEFAULT '20:10,20:30,21:10,21:30'
          );",
        (),
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS device (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            serial TEXT NOT NULL,
            forward_port INTEGER NOT NULL DEFAULT 0,
            online INTEGER NOT NULL DEFAULT 0,
            ip TEXT DEFAULT NULL,
            agent_ip TEXT NOT NULL,
            master_ip TEXT NOT NULL,
            init INTEGER NOT NULL DEFAULT 0,
            update_time TEXT DEFAULT CURRENT_TIMESTAMP
          );",
        (),
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS account (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            group_id INTEGER DEFAULT 0,
            email TEXT NOT NULL,
            pwd TEXT NOT NULL,
            fans INTEGER NOT NULL,
            shop_creator INTEGER NOT NULL DEFAULT 0,
            device TEXT DEFAULT NULL,
            register_time TEXT DEFAULT CURRENT_TIMESTAMP,
            last_login_time TEXT DEFAULT CURRENT_TIMESTAMP
          );",
        (),
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS material (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            group_id INTEGER  DEFAULT 0,
            name TEXT NOT NULL,
            md5 TEXT NOT NULL,
            used INTEGER NOT NULL DEFAULT 0,
            create_time TEXT DEFAULT CURRENT_TIMESTAMP
          );",
        (),
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS publish_job (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            group_id INTEGER  DEFAULT 0,
            material TEXT NOT NULL,
            account TEXT NOT NULL,
            title TEXT DEFAULT NULL,
            tags TEXT DEFAULT NULL,
            status INTEGER NOT NULL DEFAULT 0,
            start_time TEXT DEFAULT CURRENT_TIMESTAMP,
            end_time TEXT DEFAULT CURRENT_TIMESTAMP,
            publish_type INTEGER NOT NULL DEFAULT 1,
            product_link TEXT DEFAULT NULL,
            create_time TEXT DEFAULT CURRENT_TIMESTAMP
          );",
        (),
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS train_job (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            group_id INTEGER  DEFAULT 0,
            account TEXT NOT NULL,
            click INTEGER  DEFAULT 0,
            follow INTEGER  DEFAULT 0,
            favorites INTEGER  DEFAULT 0,
            status INTEGER NOT NULL DEFAULT 0,
            start_time TEXT DEFAULT CURRENT_TIMESTAMP,
            end_time TEXT DEFAULT CURRENT_TIMESTAMP,
            create_time TEXT DEFAULT CURRENT_TIMESTAMP
          );",
        (),
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS dialog_watcher (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        conditions TEXT NOT NULL,
        action TEXT NOT NULL,
        status INTEGER NOT NULL DEFAULT 0,
        create_time TEXT DEFAULT CURRENT_TIMESTAMP
      );",
        (),
    )?;

    let count: i64 = conn.query_row("SELECT COUNT(*) FROM dialog_watcher", [], |row| row.get(0))?;

    if count == 0 {
        //insert init data to dialog_watcher
        log::info!("insert init data to dialog_watcher");

        conn.execute(
        "INSERT INTO dialog_watcher (name, conditions, action,status) VALUES ('init1', 'DENY,ALLOW', 'click',1);",
        (),
      )?;
    }

    Ok(())
}
