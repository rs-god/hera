use autometrics::autometrics;
use log::info;
use logger::Logger;
use monitor::metrics::{API_SLO, prometheus_init};
use std::net::SocketAddr;
use std::process;
use std::time::Duration;
// 引入axum相关包
use axum::{Json, Router, http::StatusCode, response::IntoResponse, routing::get};
use serde::{Deserialize, Serialize};
use shutdown::graceful_shutdown;
use tokio::net::TcpListener;

// cargo run --example metrics_basic
// 如果想在启动时改变日志级别，可以通过指定环境变量启动应用
// 日志level 优先级  error > warn > info > debug > trace
// 启动方式：RUST_LOG=info cargo run --example metrics_basic
#[tokio::main]
async fn main() {
    Logger::new().init();
    info!("current process pid:{}", process::id());
    info!("service start...");

    // build http /metrics endpoint
    let metrics_port = 8090;
    let metrics_server = prometheus_init(metrics_port);
    let metrics_handler = tokio::spawn(metrics_server);

    let app_port = 8080;
    // http handler
    let http_handler = tokio::spawn(async move {
        let address: SocketAddr = format!("0.0.0.0:{}", app_port).parse().unwrap();

        info!("http server run on:{}", address.to_string());

        // Create axum router
        let router = api_router();

        // Create a `TcpListener` using tokio.
        let listener = TcpListener::bind(address).await.unwrap();

        // Run the server with graceful shutdown
        let graceful_wait_time = 5;
        axum::serve(listener, router)
            .with_graceful_shutdown(graceful_shutdown(Duration::from_secs(graceful_wait_time)))
            .await
            .expect("failed to start gateway service");
    });

    // start http gateway and metrics service
    let _ = tokio::try_join!(http_handler, metrics_handler)
        .expect("failed to start http gateway and metrics service");
}

// create api router
pub fn api_router() -> Router {
    // set api group and not found handler for api/xxx
    let api_routers = Router::new()
        .route("/", get(root))
        .route("/home", get(home))
        .fallback(api_not_found);

    let router = Router::new()
        .nest("/api", api_routers)
        .route("/", get(root))
        .fallback(not_found_handler);

    router
}

#[autometrics]
pub async fn root() -> &'static str {
    "Hello, World!"
}

#[autometrics(objective = API_SLO)]
// 也可以使用下面的方式，简单处理
// #[autometrics]
pub async fn home() -> &'static str {
    "Hello, home!"
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Reply<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
}

// empty object,like {}
#[derive(Deserialize, Serialize, Debug)]
pub struct EmptyObject {}

// api handler not found
async fn api_not_found() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(Reply {
            code: 404,
            message: "api not found".to_string(),
            data: Some(EmptyObject {}),
        }),
    )
}

// handler not found for global router not found
async fn not_found_handler() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "this page not found")
}
