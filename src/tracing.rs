use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::Error;
use anyhow::Context;
use opentelemetry::sdk::trace::Tracer;
use opentelemetry::sdk::Resource;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use tonic::metadata::MetadataMap;
use tracing::Span;
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder, TracingLogger};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::registry::LookupSpan;

pub struct LedgerRootSpanBuilder;

impl RootSpanBuilder for LedgerRootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span {
        tracing_actix_web::root_span!(request, user_id = tracing::field::Empty)
    }

    fn on_request_end<B: actix_web::body::MessageBody>(
        span: Span,
        outcome: &Result<ServiceResponse<B>, Error>,
    ) {
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}

pub fn create_middleware() -> TracingLogger<LedgerRootSpanBuilder> {
    TracingLogger::<LedgerRootSpanBuilder>::new()
}

pub fn create_opentelemetry_layer<S>(
    service_name: &'static str,
    honeycomb_api_key: &str,
) -> Result<OpenTelemetryLayer<S, Tracer>, anyhow::Error>
where
    S: tracing::Subscriber + for<'span> LookupSpan<'span>,
{
    let mut metadata_map = MetadataMap::with_capacity(1);
    metadata_map.insert("x-honeycomb-team", honeycomb_api_key.parse().unwrap());
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint("https://api.honeycomb.io")
        .with_metadata(metadata_map);

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(
            opentelemetry::sdk::trace::config().with_resource(Resource::new(vec![KeyValue::new(
                "service.name",
                service_name,
            )])),
        )
        .with_exporter(exporter)
        .install_simple()
        .context("Unable to create tracer")?;
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);
    Ok(telemetry_layer)
}
