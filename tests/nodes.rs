use std::{
    fs,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sea_orm::{prelude::DateTime, DatabaseBackend, MockDatabase, MockExecResult};
use telemetry_service::{entities::node, Server};
use tower::ServiceExt;

use test_log::test;

const MOCK_SOCKET_ADDRESS: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

#[test(tokio::test)]
async fn wrong_path() {
    let db_mainnet = MockDatabase::new(DatabaseBackend::Postgres).into_connection();
    let db_testnet = MockDatabase::new(DatabaseBackend::Postgres).into_connection();

    let server = Server::new(MOCK_SOCKET_ADDRESS, db_mainnet, db_testnet).unwrap();
    let app = server.app();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/asdf/mainnt")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[test(tokio::test)]
async fn wrong_method() {
    wrong_method_impl("/nodes/mainnet").await;
    wrong_method_impl("/nodes/testnet").await;
}

async fn wrong_method_impl(uri: &str) {
    let db_mainnet = MockDatabase::new(DatabaseBackend::Postgres).into_connection();
    let db_testnet = MockDatabase::new(DatabaseBackend::Postgres).into_connection();

    let server = Server::new(MOCK_SOCKET_ADDRESS, db_mainnet, db_testnet).unwrap();

    let app = server.app();
    let response = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[test(tokio::test)]
async fn non_utf8_error() {
    non_utf8_error_impl("/nodes/mainnet").await;
    non_utf8_error_impl("/nodes/testnet").await;
}

async fn non_utf8_error_impl(uri: &str) {
    let db_mainnet = MockDatabase::new(DatabaseBackend::Postgres).into_connection();
    let db_testnet = MockDatabase::new(DatabaseBackend::Postgres).into_connection();

    let server = Server::new(MOCK_SOCKET_ADDRESS, db_mainnet, db_testnet).unwrap();
    let app = server.app();

    let invalid_utf8 = vec![255];
    let response = app
        .oneshot(
            Request::builder()
                .uri(uri)
                .method("POST")
                .body(Body::from(invalid_utf8))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test(tokio::test)]
async fn wrong_input_error() {
    wrong_input_error_impl("/nodes/mainnet").await;
    wrong_input_error_impl("/nodes/testnet").await;
}

async fn wrong_input_error_impl(uri: &str) {
    let db_mainnet = MockDatabase::new(DatabaseBackend::Postgres).into_connection();
    let db_testnet = MockDatabase::new(DatabaseBackend::Postgres).into_connection();

    let server = Server::new(MOCK_SOCKET_ADDRESS, db_mainnet, db_testnet).unwrap();
    let app = server.app();

    let invalid_json = "InvalidUnparsableJSON{{}:;;;";
    let response = app
        .oneshot(
            Request::builder()
                .uri(uri)
                .method("POST")
                .body(Body::from(invalid_json))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[test(tokio::test)]
async fn entity_insert() {
    // Values for mock node and exec don't really matter.
    let mock_node = node::Model {
        id: String::new(),
        account_id: None,
        last_seen: DateTime::default(),
        last_height: 0,
        last_hash: String::new(),
        agent_name: String::new(),
        agent_version: String::new(),
        agent_build: String::new(),
        peer_count: 0,
        is_validator: false,
        status: String::new(),
        bandwidth_download: 0,
        bandwidth_upload: 0,
        cpu_usage: 0.0,
        memory_usage: 0,
        boot_time_seconds: 0,
        block_production_tracking_delay: 0.0,
        min_block_production_delay: 0.0,
        max_block_production_delay: 0.0,
        max_block_wait_delay: 0.0,
        chain_id: None,
        protocol_version: None,
    };
    let mock_exec = MockExecResult {
        last_insert_id: 1,
        rows_affected: 1,
    };

    let db_mainnet = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![mock_node.clone()]])
        .append_exec_results([mock_exec.clone()])
        .into_connection();
    let db_testnet = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![mock_node.clone()]])
        .append_exec_results([mock_exec.clone()])
        .into_connection();

    let json = fs::read_to_string("res/example_telemetry_payload_v1").unwrap();
    let server = Server::new(MOCK_SOCKET_ADDRESS, db_mainnet, db_testnet).unwrap();

    let app = server.app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/nodes/mainnet")
                .method("POST")
                .body(Body::from(json.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let app = server.app();
    let response = app
        .oneshot(
            Request::builder()
                .uri("/nodes/testnet")
                .method("POST")
                .body(Body::from(json.clone()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let (db_mainnet, db_testnet) = server.into_db_connections();
    assert_eq!(db_mainnet.unwrap().into_transaction_log().len(), 1,);
    assert_eq!(db_testnet.unwrap().into_transaction_log().len(), 1,);
}
