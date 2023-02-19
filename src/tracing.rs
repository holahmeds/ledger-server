use crate::config::HoneycombConfig;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::Error;
use std::future::{ready, Future, Ready};
use std::pin::Pin;
use tracing::Span;
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder, TracingLogger};
use tracing_honeycomb::{HoneycombTelemetry, LibhoneyReporter, SpanId, TelemetryLayer, TraceId};

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
