use crate::config::{Config, HoneycombConfig};
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use anyhow::Context;
use opentelemetry::sdk::trace::Tracer;
use opentelemetry::sdk::Resource;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use std::future::{ready, Future, Ready};
use std::pin::Pin;
use tonic::metadata::MetadataMap;
use tracing::Span;
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder, TracingLogger};
use tracing_honeycomb::{HoneycombTelemetry, LibhoneyReporter, SpanId, TelemetryLayer, TraceId};
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

pub fn create_telemetry_layer(
    service_name: &'static str,
    config: HoneycombConfig,
) -> TelemetryLayer<HoneycombTelemetry<LibhoneyReporter>, SpanId, TraceId> {
    let honeycomb_config = libhoney::Config {
        options: libhoney::client::Options {
            api_key: config.api_key,
            dataset: config.dataset,
            ..libhoney::client::Options::default()
        },
        transmission_options: libhoney::transmission::Options::default(),
    };
    tracing_honeycomb::new_honeycomb_telemetry_layer(service_name, honeycomb_config)
}

pub struct TelemetryMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for TelemetryMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        tracing_honeycomb::register_dist_tracing_root(TraceId::new(), None)
            .expect("honeycomb tracing layer registered");
        Box::pin(self.service.call(req))
    }
}

pub struct Telemetry;

impl<S, B> Transform<S, ServiceRequest> for Telemetry
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = TelemetryMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TelemetryMiddleware { service }))
    }
}

pub fn create_opentelemetry_layer<S>(
    config: &Config,
    service_name: &'static str,
) -> Result<OpenTelemetryLayer<S, Tracer>, anyhow::Error>
where
    S: tracing::Subscriber + for<'span> LookupSpan<'span>,
{
    let mut metadata_map = MetadataMap::with_capacity(1);
    metadata_map.insert(
        "x-honeycomb-team",
        config.honeycomb.api_key.parse().unwrap(),
    );
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
