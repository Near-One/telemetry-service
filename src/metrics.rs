use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use derive_more::Constructor;
use prometheus_client::encoding::EncodeLabelSet;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::histogram::Histogram;
use prometheus_client::registry::Unit;
use prometheus_client::{encoding::text::encode, registry::Registry};
use std::sync::Arc;

use crate::server::ServerState;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet, Constructor)]
pub struct Labels {
    network: String,
}

pub struct Metrics {
    pub total_requests: Family<Labels, Counter>,
    pub successful_requests: Family<Labels, Counter>,
    pub failed_requests: Family<Labels, Counter>,
    pub request_latency: Family<Labels, Histogram>,
}

pub(crate) async fn metric_handler(state: State<ServerState>) -> impl IntoResponse {
    let mut buf = String::new();
    match encode(&mut buf, &state.metrics_registry) {
        Ok(()) => (
            [(
                header::CONTENT_TYPE,
                "text/plain; version=0.0.4; charset=utf-8",
            )],
            buf,
        )
            .into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

pub(crate) fn create_registry_and_metrics() -> (Arc<Registry>, Arc<Metrics>) {
    let mut registry = <Registry>::with_prefix("telemetry_service");

    let total_requests = Family::<Labels, Counter>::default();
    registry.register(
        "total_requests",
        "Number of total requests",
        total_requests.clone(),
    );
    let successful_requests = Family::<Labels, Counter>::default();
    registry.register(
        "successful_requests",
        "Number of successful requests",
        successful_requests.clone(),
    );
    let failed_requests = Family::<Labels, Counter>::default();
    registry.register(
        "failed_requests",
        "Number of failed requests",
        failed_requests.clone(),
    );
    let request_latency = Family::<Labels, Histogram>::new_with_constructor(|| {
        let buckets = [
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ];
        Histogram::new(buckets.into_iter())
    });
    registry.register_with_unit(
        "request_latency",
        "Request latency",
        Unit::Seconds,
        request_latency.clone(),
    );

    let metrics = Metrics {
        total_requests,
        successful_requests,
        failed_requests,
        request_latency,
    };
    (Arc::new(registry), Arc::new(metrics))
}
