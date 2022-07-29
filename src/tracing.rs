use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::Error;
use tracing::Span;
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder, TracingLogger};

pub struct LedgerRootSpanBuilder;

impl RootSpanBuilder for LedgerRootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span {
        tracing_actix_web::root_span!(request, user_id = tracing::field::Empty)
    }

    fn on_request_end<B>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}

pub fn create_middleware() -> TracingLogger<LedgerRootSpanBuilder> {
    TracingLogger::<LedgerRootSpanBuilder>::new()
}
