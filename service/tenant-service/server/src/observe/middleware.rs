use opentelemetry::trace::TraceContextExt as _;
use tracing_opentelemetry::OpenTelemetrySpanExt as _;

use crate::observe::LOG_LEVEL;

pub fn trace_layer() -> tower_http::trace::TraceLayer<
    MakeClassify,
    OpentelemetryMakeSpan,
    OpentelemetryOnRequest,
    tower_http::trace::DefaultOnResponse,
    tower_http::trace::DefaultOnBodyChunk,
    tower_http::trace::DefaultOnEos,
    OpentelemetryOnFailure,
> {
    tower_http::trace::TraceLayer::new(MakeClassify)
        .make_span_with(OpentelemetryMakeSpan)
        .on_request(OpentelemetryOnRequest)
        .on_failure(OpentelemetryOnFailure)
}

#[derive(Copy, Clone)]
pub struct MakeClassify;

impl tower_http::classify::MakeClassifier for MakeClassify {
    type Classifier = Classifier;
    type FailureClass = FailureClass;
    // TODO: ClassifyEos を定義する
    type ClassifyEos = tower_http::classify::NeverClassifyEos<FailureClass>;

    fn make_classifier<B>(&self, _: &http::Request<B>) -> Self::Classifier {
        Classifier
    }
}

#[derive(Copy, Clone)]
pub struct Classifier;

impl tower_http::classify::ClassifyResponse for Classifier {
    type FailureClass = FailureClass;
    // TODO: ClassifyEos を定義する
    type ClassifyEos = tower_http::classify::NeverClassifyEos<FailureClass>;

    fn classify_response<B>(
        self,
        res: &http::Response<B>,
    ) -> tower_http::classify::ClassifiedResponse<Self::FailureClass, Self::ClassifyEos> {
        let status = tonic::Status::from_header_map(res.headers());
        if let Some(status) = status {
            let f = FailureClass {
                code: status.code(),
                message: status.message().to_string(),
            };
            tower_http::classify::ClassifiedResponse::Ready(Err(f))
        } else {
            tower_http::classify::ClassifiedResponse::Ready(Ok(()))
        }
    }

    fn classify_error<E>(self, error: &E) -> Self::FailureClass
    where
        E: std::fmt::Display + 'static,
    {
        Self::FailureClass {
            code: tonic::Code::Unknown,
            message: error.to_string(),
        }
    }
}

pub trait FailureClassExt {
    fn code(&self) -> tonic::Code;
    fn message(&self) -> String;
}

pub struct FailureClass {
    code: tonic::Code,
    message: String,
}

impl FailureClassExt for FailureClass {
    fn code(&self) -> tonic::Code {
        self.code
    }

    fn message(&self) -> String {
        self.message.clone()
    }
}

#[derive(Clone)]
pub struct OpentelemetryMakeSpan;

impl<B> tower_http::trace::MakeSpan<B> for OpentelemetryMakeSpan {
    fn make_span(&mut self, req: &http::Request<B>) -> tracing::Span {
        let span = if req.uri().path()
            == "/grpc.reflection.v1alpha.ServerReflection/ServerReflectionInfo"
        {
            tracing::debug_span!(
                "",
                otel.name = %req.uri().path(),
                rpc.system = "grpc",
                rpc.method = tracing::field::Empty,
                rpc.service = tracing::field::Empty,
                rpc.grpc.full_method = tracing::field::Empty,
                rpc.grpc.status_code = tracing::field::Empty,
                rpc.grpc.message = tracing::field::Empty,
            )
        } else {
            tracing::span!(
                LOG_LEVEL,
                "",
                otel.name = %req.uri().path(),
                rpc.system = "grpc",
                rpc.method = tracing::field::Empty,
                rpc.service = tracing::field::Empty,
                rpc.grpc.full_method = tracing::field::Empty,
                rpc.grpc.status_code = tracing::field::Empty,
                rpc.grpc.message = tracing::field::Empty,
            )
        };

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
        let full_method = req.uri().path();

        span.record("rpc.grpc.full_method", full_method);

        let methods: Vec<_> = full_method.split("/").collect();
        if let (Some(service), Some(method)) = (methods.get(1), methods.get(2)) {
            span.record(
                opentelemetry_semantic_conventions::trace::RPC_SERVICE.as_str(),
                service,
            );
            span.record(
                opentelemetry_semantic_conventions::trace::RPC_METHOD.as_str(),
                method,
            );
        }
    }
}

#[derive(Clone)]
pub struct OpentelemetryOnFailure;

impl<F> tower_http::trace::OnFailure<F> for OpentelemetryOnFailure
where
    F: FailureClassExt,
{
    fn on_failure(
        &mut self,
        failure_classification: F,
        _latency: std::time::Duration,
        span: &tracing::Span,
    ) {
        tracing::error!("{}", failure_classification.message());
        span.record(
            opentelemetry_semantic_conventions::trace::RPC_GRPC_STATUS_CODE.as_str(),
            failure_classification.code().to_string(),
        );
        span.record("rpc.grpc.message", failure_classification.message());
    }
}
