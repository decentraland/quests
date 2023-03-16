use actix_web_prom::{PrometheusMetrics, PrometheusMetricsBuilder};

pub fn metrics() -> PrometheusMetrics {
    PrometheusMetricsBuilder::new("dcl_quests")
        .endpoint("/metrics")
        .build()
        .unwrap()
}
