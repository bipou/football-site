use crate::utils::constant;
use std::sync::OnceLock;
use surrealdb::Surreal;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;

static DB: OnceLock<Surreal<Client>> = OnceLock::new();

pub async fn init() {
    let db_url = constant::config().db_url.clone();
    let db_ns = constant::config().db_ns.clone();
    let db_name = constant::config().db_name.clone();
    let db_user = constant::config().db_user.clone();
    let db_pass = constant::config().db_pass.clone();

    let db = Surreal::new::<Ws>(&db_url)
        .await
        .unwrap_or_else(|e| panic!("connect {db_url}: {e}"));

    db.signin(Root {
        username: db_user,
        password: db_pass,
    })
    .await
    .unwrap_or_else(|e| panic!("auth: {e}"));

    db.use_ns(&db_ns)
        .await
        .unwrap_or_else(|e| panic!("ns: {e}"));
    db.use_db(&db_name)
        .await
        .unwrap_or_else(|e| panic!("db: {e}"));

    DB.set(db).expect("DB already set");

    leptos::logging::log!("SurrealDB ready: {db_url} ns={db_ns} db={db_name}");
}

pub fn get_db() -> &'static Surreal<Client> {
    DB.get().expect("db::init() not called")
}
