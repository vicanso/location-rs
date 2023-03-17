use axum::{error_handling::HandleErrorLayer, middleware::from_fn, Json, Router};
use axum_client_ip::SecureClientIpSource;
use axum_extra::routing::{RouterExt, TypedPath};
use error::HTTPResult;
use serde::Deserialize;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::signal;
use tower::ServiceBuilder;
use tracing::info;

mod error;
mod gen;
mod ip;
mod ip_data;
mod middleware;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    if let Some(arg) = std::env::args().nth(1) {
        if arg == "build" {
            let count = std::env::args()
                .nth(2)
                .unwrap_or_default()
                .parse::<i64>()
                .unwrap_or_default();
            gen::generate_ip_data(count);
            return;
        }
    }

    let app = Router::new()
        .typed_get(get_location)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(error::handle_error))
                .timeout(Duration::from_secs(30)),
        )
        // 后面的layer先执行
        .layer(from_fn(middleware::access_log))
        .layer(from_fn(middleware::entry))
        .layer(SecureClientIpSource::ConnectInfo.into_extension());

    let addr = "127.0.0.1:7001".parse().unwrap();
    info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("signal received, starting graceful shutdown");
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/ip-locations/:ip")]
struct IPParams {
    ip: String,
}

async fn get_location(IPParams { ip }: IPParams) -> HTTPResult<Json<ip::Location>> {
    let data = ip::get_location(&ip)?;
    Ok(Json(data))
}
