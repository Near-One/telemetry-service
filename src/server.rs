use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{routing::get, Router};
use derive_more::Constructor;
use prometheus_client::registry::Registry;
use sea_orm::DatabaseConnection;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::timeout::TimeoutLayer;
use tracing::info;

use crate::health::health_handler;
use crate::metrics::{create_registry_and_metrics, metric_handler, Metrics};
use crate::nodes::{nodes_handler_mainnet, nodes_handler_testnet, ChainId};
use crate::Error;

pub struct Server {
    address: SocketAddr,
    state: ServerState,
}

#[derive(Constructor, Clone)]
pub(crate) struct ServerState {
    pub(crate) metrics_registry: Arc<Registry>,
    pub(crate) metrics: Arc<Metrics>,
    pub(crate) db_mainnet: Arc<DatabaseConnection>,
    pub(crate) db_testnet: Arc<DatabaseConnection>,
}

impl ServerState {
    pub(crate) fn database(&self, chain: ChainId) -> &Arc<DatabaseConnection> {
        match chain {
            ChainId::Mainnet => &self.db_mainnet,
            ChainId::Testnet => &self.db_testnet,
        }
    }
}

impl Server {
    pub fn new(
        address: SocketAddr,
        db_mainnet: DatabaseConnection,
        db_testnet: DatabaseConnection,
    ) -> Result<Self, Error> {
        let (metrics_registry, metrics) = create_registry_and_metrics();
        Ok(Self {
            address,
            state: ServerState::new(
                metrics_registry,
                metrics,
                Arc::new(db_mainnet),
                Arc::new(db_testnet),
            ),
        })
    }

    pub async fn run(&self) -> Result<(), Error> {
        info!("starting HTTP server on {}", self.address);

        let listener = TcpListener::bind(self.address).await?;
        let app = self.app();
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await?;
        Ok(())
    }

    pub fn app(&self) -> Router {
        Router::new()
            .route("/metrics", get(metric_handler))
            .route("/healthz", get(health_handler))
            .route("/nodes/mainnet", post(nodes_handler_mainnet))
            .route("/nodes/testnet", post(nodes_handler_testnet))
            .layer((
                // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
                // requests don't hang forever.
                TimeoutLayer::new(Duration::from_secs(10)),
            ))
            .with_state(self.state.clone())
            .fallback(handler_404)
    }

    pub fn into_db_connections(self) -> (Option<DatabaseConnection>, Option<DatabaseConnection>) {
        (
            Arc::<DatabaseConnection>::into_inner(self.state.db_mainnet),
            (Arc::<DatabaseConnection>::into_inner(self.state.db_testnet)),
        )
    }
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
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}
