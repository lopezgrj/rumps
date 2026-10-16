mod application;
mod infrastructure;
mod presentation;

use application::AppConfig;
use sqlx::{AnyPool, any::AnyPoolOptions};
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    sqlx::any::install_default_drivers();

    let config = AppConfig::from_env()?;
    let pool = connect_database(&config.database).await?;
    infrastructure::migrations::migrate(&pool, config.database.kind).await?;
    let authenticator = match &config.oidc {
        Some(oidc_config) => Some(Arc::new(
            infrastructure::oidc::OidcAuthenticator::load(oidc_config).await?,
        )),
        None => None,
    };
    let listener = TcpListener::bind(config.bind_addr).await?;

    tracing::info!(
        address = %listener.local_addr()?,
        database = %config.database.kind,
        oidc_enabled = authenticator.is_some(),
        "RUMPS API listening"
    );

    axum::serve(listener, presentation::router(pool, authenticator))
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn connect_database(config: &application::DatabaseConfig) -> Result<AnyPool, sqlx::Error> {
    AnyPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.url)
        .await
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!(%error, "failed to listen for Ctrl-C");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => tracing::error!(%error, "failed to listen for terminate signal"),
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
