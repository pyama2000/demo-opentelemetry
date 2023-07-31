use tracing_opentelemetry::OpenTelemetrySpanExt as _;

#[derive(Debug)]
pub struct Client(reqwest_middleware::ClientWithMiddleware);

impl Client {
    pub fn new() -> Self {
        let builder = reqwest_middleware::ClientBuilder::new(reqwest::Client::new())
            .with(reqwest_tracing::TracingMiddleware::<
                reqwest_tracing::SpanBackendWithUrl,
            >::new())
            .build();
        Self(builder)
    }

    pub fn request<U: reqwest::IntoUrl>(
        &self,
        method: http::Method,
        url: U,
        span: tracing::Span,
    ) -> reqwest_middleware::RequestBuilder {
        let mut headers = http::HeaderMap::new();
        opentelemetry::global::get_text_map_propagator(|p| {
            p.inject_context(
                &span.context(),
                &mut opentelemetry_http::HeaderInjector(&mut headers),
            )
        });
        self.0.request(method, url).headers(headers)
    }
}
