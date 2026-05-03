use mongodb::Database;
use std::sync::OnceLock;

pub static DB: OnceLock<Database> = OnceLock::new();

pub fn get_db() -> &'static Database {
    DB.get()
        .expect("Database not initialized — call DB.set() in main before serving")
}
