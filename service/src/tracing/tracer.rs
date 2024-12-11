use opentelemetry::global;
use opentelemetry_sdk::{
    propagation::TraceContextPropagator, runtime::Tokio, trace::TracerProvider,
};
use std::error::Error;
use tracing_subscriber::fmt::format::{Format, JsonFields};
use tracing_subscriber::{prelude::*, EnvFilter};

pub struct Tracer;

impl Tracer {
    pub fn install_stdout() -> Result<(), Box<dyn Error + Send + Sync>> {
        global::set_text_map_propagator(TraceContextPropagator::new());

        use opentelemetry_stdout::SpanExporter;
        let provider = TracerProvider::builder()
            .with_batch_exporter(SpanExporter::default(), Tokio)
            .build();
        global::set_tracer_provider(provider);

        let layer = tracing_subscriber::fmt::layer()
            .event_format(Format::default().pretty())
            .fmt_fields(JsonFields::new())
            .with_filter(EnvFilter::from_default_env());

        tracing_subscriber::registry().with(layer).init();

        Ok(())
    }
}
