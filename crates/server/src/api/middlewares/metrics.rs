use actix_web_prom::{PrometheusMetrics, PrometheusMetricsBuilder};

pub fn metrics() -> PrometheusMetrics {
    PrometheusMetricsBuilder::new("api")
        .endpoint("/metrics")
        .build()
        .unwrap()
}
