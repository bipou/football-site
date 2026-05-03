use std::sync::OnceLock;
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;

static DB: OnceLock<Surreal<Client>> = OnceLock::new();

pub async fn init() {
    let db_url  = std::env::var("DB_URL").expect("DB_URL");
    let db_ns   = std::env::var("DB_NS").unwrap_or_else(|_| "football".into());
    let db_name = std::env::var("DB_NAME").unwrap_or_else(|_| "football".into());
    let db_user = std::env::var("DB_USER").expect("DB_USER");
    let db_pass = std::env::var("DB_PASS").expect("DB_PASS");

    leptos::logging::log!("SurrealDB {db_url} ns={db_ns} db={db_name}");

    let db = Surreal::new::<Ws>(&db_url).await
        .unwrap_or_else(|e| panic!("connect {db_url}: {e}"));

    db.signin(Root { username: &db_user, password: &db_pass }).await
        .unwrap_or_else(|e| panic!("auth: {e}"));

    db.use_ns(&db_ns).await.unwrap_or_else(|e| panic!("ns: {e}"));
    db.use_db(&db_name).await.unwrap_or_else(|e| panic!("db: {e}"));

    DB.set(db).expect("DB already set");
    leptos::logging::log!("SurrealDB ready");
}

pub fn get_db() -> &'static Surreal<Client> {
    DB.get().expect("db::init() not called")
}
