#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use tower_http::services::ServeDir;

    use football_site::app::{shell, App};
    use football_site::server::db::DB;

    dotenvy::dotenv().ok();

    // ── MongoDB ──────────────────────────────────────────────────────────────
    let mongo_uri  = std::env::var("MONGODB_URI").expect("MONGODB_URI must be set");
    let mongo_name = std::env::var("MONGODB_NAME").expect("MONGODB_NAME must be set");
    let client = mongodb::Client::with_uri_str(&mongo_uri)
        .await
        .expect("Failed to connect to MongoDB");
    DB.set(client.database(&mongo_name)).expect("DB already set");

    // ── Leptos ───────────────────────────────────────────────────────────────
    let conf           = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr           = leptos_options.site_addr;
    let routes         = generate_route_list(App);

    let app = Router::new()
        .route("/api/{*fn_name}", axum::routing::post(leptos_axum::handle_server_fns))
        .leptos_routes(&leptos_options, routes, {
            let opts = leptos_options.clone();
            move || shell(opts.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .nest_service("/public", ServeDir::new("public"))
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    leptos::logging::log!("BiPou football-site listening on http://{addr}");
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
