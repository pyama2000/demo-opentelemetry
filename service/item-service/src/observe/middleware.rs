use opentelemetry::trace::TraceContextExt as _;
use tracing_opentelemetry::OpenTelemetrySpanExt as _;

use crate::observe::LOG_LEVEL;

pub fn trace_layer() -> tower_http::trace::TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
    OpentelemetryMakeSpan,
    OpentelemetryOnRequest,
    OpentelemetryOnResponse,
> {
    tower_http::trace::TraceLayer::new_for_http()
        .make_span_with(OpentelemetryMakeSpan)
        .on_request(OpentelemetryOnRequest)
        .on_response(OpentelemetryOnResponse)
}

#[derive(Clone)]
pub struct OpentelemetryMakeSpan;

impl<B> tower_http::trace::MakeSpan<B> for OpentelemetryMakeSpan {
    fn make_span(&mut self, req: &http::Request<B>) -> tracing::Span {
        let span = tracing::span!(
            LOG_LEVEL,
            "",
            otel.name = %req.uri(),
            span.kind = "server",
            http.method = tracing::field::Empty,
            http.status_code = tracing::field::Empty,
        );

        let parent_cx = opentelemetry::global::get_text_map_propagator(|p| {
            p.extract(&opentelemetry_http::HeaderExtractor(req.headers()))
        });
        if parent_cx.span().span_context().is_valid() {
            span.set_parent(parent_cx);
        }

        span
    }
}

#[derive(Clone)]
pub struct OpentelemetryOnRequest;

impl<B> tower_http::trace::OnRequest<B> for OpentelemetryOnRequest {
    fn on_request(&mut self, req: &http::Request<B>, span: &tracing::Span) {
        span.record(
            opentelemetry_semantic_conventions::trace::HTTP_METHOD.as_str(),
            &tracing::field::display(req.method()),
        );
    }
}

#[derive(Clone)]
pub struct OpentelemetryOnResponse;

impl<B> tower_http::trace::OnResponse<B> for OpentelemetryOnResponse {
    fn on_response(
        self,
        res: &http::Response<B>,
        _latency: std::time::Duration,
        span: &tracing::Span,
    ) {
        span.record(
            opentelemetry_semantic_conventions::trace::HTTP_STATUS_CODE.as_str(),
            res.status().as_str(),
        );
    }
}
