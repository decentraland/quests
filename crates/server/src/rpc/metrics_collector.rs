use actix_web::cookie::time::Instant;
use prometheus::{
    Encoder, HistogramOpts, HistogramVec, IntCounterVec, IntGauge, Opts, Registry, TextEncoder,
};

pub struct MetricsCollector {
    registry: Registry,
    procedure_call_collector: IntCounterVec,
    procedure_call_duration_collector: HistogramVec,
    connections_collector: IntGauge,
}

impl MetricsCollector {
    pub fn new() -> Self {
        let registry = Registry::new();

        let procedure_call_collector = IntCounterVec::new(
            Opts::new(
                "dcl_quests_rpc_procedure_call_total",
                "DCL Quests RPC Calls",
            ),
            &["procedure", "status"],
        )
        .expect("expect to be able to create a custom collector");

        let procedure_call_duration_collector = HistogramVec::new(
            HistogramOpts::new(
                "dcl_quests_rpc_procedure_call_duration_seconds",
                "DCL Quests RPC Calls Duration in Seconds",
            ),
            &["procedure", "status"],
        )
        .expect("expect to be able to create a custom collector");

        let connections_collector = IntGauge::new(
            "dcl_quests_ws_connected_clients_total",
            "DCL Quests WS connected clients",
        )
        .expect("expect to be able to create a custom collector");

        registry
            .register(Box::new(procedure_call_collector.clone()))
            .expect("expect to be able to register a custom collector");
        registry
            .register(Box::new(procedure_call_duration_collector.clone()))
            .expect("expect to be able to register a custom collector");
        registry
            .register(Box::new(connections_collector.clone()))
            .expect("expect to be able to register a custom collector");

        Self {
            registry,
            procedure_call_collector,
            connections_collector,
            procedure_call_duration_collector,
        }
    }

    pub fn record_procedure_call<'a, P: Into<&'a str>, S: Into<&'a str>>(
        &self,
        procedure: P,
        status: S,
    ) {
        self.procedure_call_collector
            .with_label_values(&[procedure.into(), status.into()])
            .inc()
    }

    pub fn record_procedure_call_duration<'a, P: Into<&'a str> + 'a, S: Into<&'a str> + 'a>(
        &'a self,
        procedure: P,
    ) -> impl FnOnce(S) + 'a {
        let start = Instant::now();
        move |status: S| {
            self.procedure_call_duration_collector
                .with_label_values(&[procedure.into(), status.into()])
                .observe(start.elapsed().as_seconds_f64())
        }
    }

    pub fn client_connected(&self) {
        self.connections_collector.inc()
    }

    pub fn client_disconnected(&self) {
        self.connections_collector.dec()
    }

    pub fn collect(&self) -> Result<String, String> {
        let encoder = TextEncoder::new();
        let mut buffer = vec![];

        if let Err(err) = encoder.encode(&self.registry.gather(), &mut buffer) {
            return Err(err.to_string());
        }

        match String::from_utf8(buffer) {
            Ok(metrics) => Ok(metrics),
            Err(_) => Err("Metrics corrupted".to_string()),
        }
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}
