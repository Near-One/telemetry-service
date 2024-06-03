use std::{fmt, sync::Arc};

use axum::{extract::State, http::StatusCode, response::IntoResponse};
use sea_orm::{sea_query::OnConflict, ActiveValue, DatabaseConnection, EntityTrait, Iterable};
use tokio::time::Instant;
use tracing::{debug, error, trace};

use crate::{
    entities::node::{self},
    metrics::Labels,
    server::ServerState,
    telemetry::TelemetryInfo,
    Error,
};

#[derive(Debug, Clone)]
pub enum ChainId {
    Mainnet,
    Testnet,
    Other(String),
}

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ChainId::Mainnet => write!(f, "mainnet"),
            ChainId::Testnet => write!(f, "testnet"),
            ChainId::Other(_) => write!(f, "other"),
        }
    }
}

pub(crate) async fn nodes_handler_mainnet(
    state: State<ServerState>,
    body: String,
) -> impl IntoResponse {
    nodes_handler_impl(state, body, Some(ChainId::Mainnet)).await
}

pub(crate) async fn nodes_handler_testnet(
    state: State<ServerState>,
    body: String,
) -> impl IntoResponse {
    nodes_handler_impl(state, body, Some(ChainId::Testnet)).await
}

pub(crate) async fn nodes_handler(state: State<ServerState>, body: String) -> impl IntoResponse {
    nodes_handler_impl(state, body, None).await
}

async fn nodes_handler_impl(
    state: State<ServerState>,
    body: String,
    chain_from_path: Option<ChainId>,
) -> impl IntoResponse {
    let now = Instant::now();

    trace!("chain_from_path: {chain_from_path:?}, request body: {body}");

    let telemetry: Result<TelemetryInfo, Error> =
        serde_json::from_str(&body).map_err(|err| Error::InputError(err.to_string(), body));

    let chain_from_telemetry = telemetry.as_ref().ok().and_then(|info| {
        info.chain
            .chain_id
            .as_ref()
            .map(|chain| match chain.as_str() {
                "mainnet" => ChainId::Mainnet,
                "testnet" => ChainId::Testnet,
                _ => ChainId::Other(chain.clone()),
            })
    });
    // Determine the chain-id. In order of priority:
    // 1. chain-id sent inside the json
    // 2. HTTP path
    let chain = match (chain_from_telemetry, chain_from_path) {
        (Some(chain), _) => chain,
        (None, Some(chain)) => chain,
        _ => ChainId::Other("unknown".to_string()),
    };

    debug!("received node telemetry for {chain}");

    let labels = Labels::new(chain.to_string());
    state.metrics.total_requests.get_or_create(&labels).inc();

    let result = store_telemetry(state.database(&chain), &chain, telemetry).await;

    let elapsed = now.elapsed();
    state
        .metrics
        .request_latency
        .get_or_create(&labels)
        .observe(elapsed.as_secs_f64());

    match result {
        Ok(_) => {
            state
                .metrics
                .successful_requests
                .get_or_create(&labels)
                .inc();
            debug!("telemetry request for {chain} handled correctly");
            (StatusCode::NO_CONTENT, String::new())
        }
        Err(err) => {
            state.metrics.failed_requests.get_or_create(&labels).inc();
            error!("error processing {chain} request: {err:#?}");
            match err {
                Error::InputError(_, _) => (StatusCode::BAD_REQUEST, format!("{err:#?}")),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, format!("{err:#?}")),
            }
        }
    }
}

async fn store_telemetry(
    db: Option<&Arc<DatabaseConnection>>,
    chain: &ChainId,
    telemetry: Result<TelemetryInfo, Error>,
) -> Result<(), Error> {
    let telemetry = telemetry?;

    if matches!(chain, ChainId::Other(_)) {
        debug!("persisting telemetry for chains other than mainnet and testnet is disabled");
        return Ok(());
    }

    let db: &DatabaseConnection = match db {
        Some(db) => db,
        None => return Err(Error::DatabaseNotFound),
    };

    let node = node::ActiveModel {
        id: ActiveValue::Set(telemetry.chain.node_id),
        account_id: ActiveValue::Set(telemetry.chain.account_id),
        last_seen: ActiveValue::Set(chrono::offset::Utc::now().naive_utc()),
        last_height: ActiveValue::Set(telemetry.chain.latest_block_height as i64),
        last_hash: ActiveValue::Set(telemetry.chain.latest_block_hash),
        agent_name: ActiveValue::Set(telemetry.agent.name),
        agent_version: ActiveValue::Set(telemetry.agent.version),
        agent_build: ActiveValue::Set(telemetry.agent.build),
        peer_count: ActiveValue::Set(telemetry.chain.num_peers as i64),
        is_validator: ActiveValue::Set(telemetry.chain.is_validator),
        status: ActiveValue::Set(telemetry.chain.status),
        bandwidth_download: ActiveValue::Set(telemetry.system.bandwidth_download as i64),
        bandwidth_upload: ActiveValue::Set(telemetry.system.bandwidth_upload as i64),
        cpu_usage: ActiveValue::Set(telemetry.system.cpu_usage),
        memory_usage: ActiveValue::Set(telemetry.system.memory_usage as i64),
        boot_time_seconds: ActiveValue::Set(telemetry.system.boot_time_seconds),
        block_production_tracking_delay: ActiveValue::Set(
            telemetry.chain.block_production_tracking_delay,
        ),
        min_block_production_delay: ActiveValue::Set(telemetry.chain.min_block_production_delay),
        max_block_production_delay: ActiveValue::Set(telemetry.chain.max_block_production_delay),
        max_block_wait_delay: ActiveValue::Set(telemetry.chain.max_block_wait_delay),
        chain_id: ActiveValue::Set(telemetry.chain.chain_id),
        protocol_version: ActiveValue::Set(telemetry.agent.protocol_version.map(|n| n as i32)),
    };

    let on_conflict = OnConflict::column(node::Column::Id)
        .update_columns(node::Column::iter().filter(|col| !matches!(*col, node::Column::Id)))
        .to_owned();

    node::Entity::insert(node)
        .on_conflict(on_conflict)
        .exec(db)
        .await?;

    Ok(())
}
