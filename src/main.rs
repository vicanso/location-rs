use axum::http::Uri;
use axum::{
    error_handling::HandleErrorLayer, extract::Path, middleware::from_fn, routing::get, Json,
    Router,
};
use axum_client_ip::InsecureClientIp;
use error::HTTPResult;
use std::net::SocketAddr;
use std::time::Duration;
use std::{env, str::FromStr};
use tokio::signal;
use tower::ServiceBuilder;
use tracing::info;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod dist;
mod error;
mod gen;
mod ip;
mod ip_data;
mod middleware;

fn init_logger() {
    let mut level = Level::INFO;
    if let Ok(log_level) = env::var("LOG_LEVEL") {
        if let Ok(value) = Level::from_str(log_level.as_str()) {
            level = value;
        }
    }

    let timer = tracing_subscriber::fmt::time::OffsetTime::local_rfc_3339().unwrap_or_else(|_| {
        tracing_subscriber::fmt::time::OffsetTime::new(
            time::UtcOffset::from_hms(0, 0, 0).unwrap(),
            time::format_description::well_known::Rfc3339,
        )
    });

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_timer(timer)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

#[tokio::main]
async fn run() {
    let app = Router::new()
        .route("/ping", get(ping))
        .route("/api/ip-locations/{ip}", get(get_location))
        .fallback(get(serve))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(error::handle_error))
                .timeout(Duration::from_secs(30)),
        )
        // 后面的layer先执行
        .layer(from_fn(middleware::access_log))
        .layer(from_fn(middleware::entry));

    let addr = "0.0.0.0:7001";
    info!("listening on http://{addr}/");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
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

async fn get_location(
    InsecureClientIp(client_ip): InsecureClientIp,
    Path(ip): Path<String>,
) -> HTTPResult<Json<ip::Location>> {
    // TODO 判断是否内网
    // 0.0.0.0
    let value = if ip == "0.0.0.0" {
        client_ip.to_string()
    } else {
        ip
    };
    let data = ip::get_location(&value)?;
    Ok(Json(data))
}

async fn ping() -> &'static str {
    "pong"
}

async fn serve(uri: Uri) -> dist::StaticFile {
    let mut filename = &uri.path()[1..];
    if filename.is_empty() {
        filename = "index.html";
    }
    dist::get_static_file(filename)
}

fn main() {
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
    // Because we need to get the local offset before Tokio spawns any threads, our `main`
    // function cannot use `tokio::main`.

    init_logger();
    run();
}
