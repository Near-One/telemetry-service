use std::fmt;

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

#[derive(Debug, Clone, Copy)]
pub enum ChainId {
    Mainnet,
    Testnet,
}

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ChainId::Mainnet => write!(f, "mainnet"),
            ChainId::Testnet => write!(f, "testnet"),
        }
    }
}

pub(crate) async fn nodes_handler_mainnet(
    state: State<ServerState>,
    body: String,
) -> impl IntoResponse {
    nodes_handler(state, body, ChainId::Mainnet).await
}

pub(crate) async fn nodes_handler_testnet(
    state: State<ServerState>,
    body: String,
) -> impl IntoResponse {
    nodes_handler(state, body, ChainId::Testnet).await
}

async fn nodes_handler(
    state: State<ServerState>,
    body: String,
    chain: ChainId,
) -> impl IntoResponse {
    let now = Instant::now();
    debug!("received node telemetry for {chain}");
    trace!("request body: {body}");

    let labels = Labels::new(chain.to_string());
    state.metrics.total_requests.get_or_create(&labels).inc();

    let result = parse_and_store_telemetry(state.database(chain), body).await;

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

async fn parse_and_store_telemetry(db: &DatabaseConnection, body: String) -> Result<(), Error> {
    let telemetry: TelemetryInfo =
        serde_json::from_str(&body).map_err(|err| Error::InputError(err.to_string(), body))?;

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
        chain_id: ActiveValue::Set(None),
        protocol_version: ActiveValue::Set(None),
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
