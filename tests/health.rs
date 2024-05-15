use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::{body::Body, extract::Request, http::StatusCode};
use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};
use telemetry_service::Server;
use tower::ServiceExt;

use test_log::test;

const MOCK_SOCKET_ADDRESS: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

#[test(tokio::test)]
async fn health_ok() {
    let db_mainnet = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();
    let db_testnet = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();
    let server = Server::new(MOCK_SOCKET_ADDRESS, db_mainnet, db_testnet).unwrap();
    let app = server.app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[test(tokio::test)]
async fn health_ko_mainnet() {
    let db_mainnet = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_errors(vec![])
        .into_connection();
    let db_testnet = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();
    let server = Server::new(MOCK_SOCKET_ADDRESS, db_mainnet, db_testnet).unwrap();
    let app = server.app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[test(tokio::test)]
async fn health_ko_testnet() {
    let db_mainnet = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_results([MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();
    let db_testnet = MockDatabase::new(DatabaseBackend::Postgres)
        .append_exec_errors(vec![])
        .into_connection();
    let server = Server::new(MOCK_SOCKET_ADDRESS, db_mainnet, db_testnet).unwrap();
    let app = server.app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
