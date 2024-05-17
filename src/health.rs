use axum::{extract::State, http::StatusCode, response::IntoResponse};
use sea_orm::{ConnectionTrait, DatabaseConnection, DbErr, ExecResult, Statement};
use tracing::{debug, error};

use crate::server::ServerState;

pub(crate) async fn health_handler(state: State<ServerState>) -> impl IntoResponse {
    let mainnet_result = check(&state.db_mainnet).await;
    let testnet_result = check(&state.db_testnet).await;

    if let Err(err) = mainnet_result {
        error!("mainnet database error: {err}");
        return StatusCode::INTERNAL_SERVER_ERROR;
    }
    if let Err(err) = testnet_result {
        error!("testnet database error: {err}");
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    debug!("health check: success");
    StatusCode::OK
}

async fn check(db: &DatabaseConnection) -> Result<ExecResult, DbErr> {
    db.execute(Statement::from_string(
        db.get_database_backend(),
        "SELECT 1",
    ))
    .await
}
