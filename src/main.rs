#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use budgetplaner::{app::*, backend::db::init_db_pool};
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use std::net::SocketAddr;

    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,budgetplaner=debug".into()),
        )
        .init();

    let db_pool = init_db_pool().await;

    let mut conf = get_configuration(None).unwrap();

    // Railway-Kompatibilität: Falls `PORT` gesetzt ist, auf `0.0.0.0:$PORT` lauschen
    if let Ok(port_str) = std::env::var("PORT") {
        if let Ok(port) = port_str.parse::<u16>() {
            conf.leptos_options.site_addr = SocketAddr::from(([0, 0, 0, 0], port));
        }
    }

    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            move || {
                if let Some(pool) = db_pool.clone() {
                    provide_context(pool);
                }
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    log!("Budgetplaner Server läuft auf http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // Wird für reine Client/WASM-Targets nicht genutzt (Einstiegspunkt ist `src/lib.rs::hydrate`)
}
