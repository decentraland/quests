use prometheus::{Encoder, IntCounterVec, Opts, Registry, TextEncoder};

pub struct MetricsCollector {
    registry: Registry,
    procedure_call_collector: IntCounterVec,
}

impl MetricsCollector {
    pub fn new() -> Self {
        let registry = Registry::new();

        let procedure_call_collector = IntCounterVec::new(
            Opts::new("dcl_quests_rpc_procedure_call", "DCL Quests RPC Call"),
            &["procedure", "status"],
        )
        .expect("expect to be able to create a custom collector");

        registry
            .register(Box::new(procedure_call_collector.clone()))
            .expect("expect to be able to register a custom collector");

        Self {
            registry,
            procedure_call_collector,
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
