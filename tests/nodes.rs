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

// Wrong URLs should return 404.
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

// Check that HTTP methods other than POST are rejected.
#[test(tokio::test)]
async fn wrong_method() {
    wrong_method_impl("/nodes/mainnet").await;
    wrong_method_impl("/nodes/testnet").await;
    wrong_method_impl("/nodes").await;
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

// Check that non utf8 strings are rejected.
#[test(tokio::test)]
async fn non_utf8_error() {
    non_utf8_error_impl("/nodes/mainnet").await;
    non_utf8_error_impl("/nodes/testnet").await;
    non_utf8_error_impl("/nodes").await;
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

// Unparsable JSON should be rejected.
#[test(tokio::test)]
async fn wrong_input_error() {
    wrong_input_error_impl("/nodes/mainnet").await;
    wrong_input_error_impl("/nodes/testnet").await;
    wrong_input_error_impl("/nodes").await;
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

fn mock_node_and_exec() -> (node::Model, MockExecResult) {
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
    (mock_node, mock_exec)
}

// Verify happy path for ingestion of telemetry data v1.
#[test(tokio::test)]
async fn entity_insert_v1() {
    let (mock_node, mock_exec) = mock_node_and_exec();

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

    for uri in ["/nodes/mainnet", "/nodes/testnet"] {
        let app = server.app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri(uri)
                    .method("POST")
                    .body(Body::from(json.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    let (db_mainnet, db_testnet) = server.into_db_connections();
    assert_eq!(db_mainnet.unwrap().into_transaction_log().len(), 1);
    assert_eq!(db_testnet.unwrap().into_transaction_log().len(), 1);
}

// Verify happy path for ingestion of telemetry data v2, with chain-id: testnet.
#[test(tokio::test)]
async fn entity_insert_v2_testnet() {
    let (mock_node, mock_exec) = mock_node_and_exec();

    let db_mainnet = MockDatabase::new(DatabaseBackend::Postgres).into_connection();
    let db_testnet = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![mock_node; 3]])
        .append_exec_results([mock_exec.clone(), mock_exec.clone(), mock_exec.clone()])
        .into_connection();

    let json = fs::read_to_string("res/example_telemetry_payload_v2_testnet").unwrap();
    let server = Server::new(MOCK_SOCKET_ADDRESS, db_mainnet, db_testnet).unwrap();

    for uri in ["/nodes", "/nodes/mainnet", "/nodes/testnet"] {
        let app = server.app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri(uri)
                    .method("POST")
                    .body(Body::from(json.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    let (db_mainnet, db_testnet) = server.into_db_connections();
    assert_eq!(db_mainnet.unwrap().into_transaction_log().len(), 0);
    assert_eq!(db_testnet.unwrap().into_transaction_log().len(), 3);
}

// Verify happy path for ingestion of telemetry data v2, with chain-id: mainnet.
#[test(tokio::test)]
async fn entity_insert_v2_mainnet() {
    let (mock_node, mock_exec) = mock_node_and_exec();

    let db_mainnet = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![mock_node; 3]])
        .append_exec_results([mock_exec.clone(), mock_exec.clone(), mock_exec.clone()])
        .into_connection();
    let db_testnet = MockDatabase::new(DatabaseBackend::Postgres).into_connection();

    let json = fs::read_to_string("res/example_telemetry_payload_v2_mainnet").unwrap();
    let server = Server::new(MOCK_SOCKET_ADDRESS, db_mainnet, db_testnet).unwrap();

    for uri in ["/nodes", "/nodes/mainnet", "/nodes/testnet"] {
        let app = server.app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri(uri)
                    .method("POST")
                    .body(Body::from(json.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    let (db_mainnet, db_testnet) = server.into_db_connections();
    assert_eq!(db_mainnet.unwrap().into_transaction_log().len(), 3);
    assert_eq!(db_testnet.unwrap().into_transaction_log().len(), 0);
}

// Verify happy path for ingestion of telemetry data v2, with chain-id: other.
#[test(tokio::test)]
async fn entity_insert_v2_other() {
    let (mock_node, mock_exec) = mock_node_and_exec();

    let db_mainnet = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![mock_node.clone()]])
        .append_exec_results([mock_exec.clone()])
        .into_connection();
    let db_testnet = MockDatabase::new(DatabaseBackend::Postgres)
        .append_query_results([vec![mock_node.clone()]])
        .append_exec_results([mock_exec.clone()])
        .into_connection();

    let json = fs::read_to_string("res/example_telemetry_payload_v2_other").unwrap();
    let server = Server::new(MOCK_SOCKET_ADDRESS, db_mainnet, db_testnet).unwrap();

    for uri in ["/nodes", "/nodes/mainnet", "/nodes/testnet"] {
        let app = server.app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri(uri)
                    .method("POST")
                    .body(Body::from(json.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    let (db_mainnet, db_testnet) = server.into_db_connections();
    assert_eq!(db_mainnet.unwrap().into_transaction_log().len(), 0);
    assert_eq!(db_testnet.unwrap().into_transaction_log().len(), 0);
}

// Verify corner cases for ingestion of telemetry data v2, when chain-id is missing.
#[test(tokio::test)]
async fn entity_insert_v2_backward_compatibility() {
    let (mock_node, mock_exec) = mock_node_and_exec();

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

    for uri in ["/nodes", "/nodes/mainnet", "/nodes/testnet"] {
        let app = server.app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri(uri)
                    .method("POST")
                    .body(Body::from(json.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    let (db_mainnet, db_testnet) = server.into_db_connections();
    assert_eq!(db_mainnet.unwrap().into_transaction_log().len(), 1);
    assert_eq!(db_testnet.unwrap().into_transaction_log().len(), 1);
}
