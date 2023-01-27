use axum::{routing::post, Router};

use log::info;
use std::net::SocketAddr;
use tokio::signal;
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry, fmt};

mod config;
mod github;
mod utils;
#[tokio::main]
async fn main() {
    // 设置日志
    let (log_appender, _) =
        tracing_appender::non_blocking(tracing_appender::rolling::daily("log", "webhook-log"));
    
    Registry::default()
        .with(EnvFilter::from_default_env().add_directive(Level::DEBUG.into()))
        .with(fmt::layer().pretty().with_writer(std::io::stderr))
        .with(fmt::layer().with_writer(log_appender))
        .init();

    // 路由
    let app = Router::new().route("/github", post(github::github));

    // 启动服务
    let addr = SocketAddr::from(([127, 0, 0, 1], 13400));
    info!("服务启动，监听地址: {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal_handler())
        .await
        .unwrap();
}

/// 优雅关闭
pub async fn shutdown_signal_handler() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("Ctrl+C的信号监听器启动失败");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("程序关闭的信号监听器启动失败")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            println!();
            info!("收到Ctrl+C, 开始收尾");
        },
        _ = terminate => {
            info!("收到程序关闭信号，开始收尾");
        },
    }

    info!("收尾结束，程序关闭");
}
