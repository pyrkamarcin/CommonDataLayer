use opentelemetry::{global, propagation::Injector};
use opentelemetry_http::HeaderExtractor;
use tonic::{codegen::http, Request, Status};
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub trait InterceptorType = (Fn(Request<()>) -> Result<Request<()>, Status>) + Send + Sync;

/// Method used with gRPC client to inject Span ID to gRPC metadata.
/// Used mostly in `rpc` crate in `connect()`.
/// # Example:
/// ```ignore
/// pub async fn connect(addr: String) -> Result<MyServiceClient<tonic::transport::Channel>, tonic::transport::Error> {
///     let conn = crate::open_channel(addr).await?;
///
///     Ok(MyServiceClient::with_interceptor(conn, &tracing_utils::grpc::interceptor))
/// }
/// ```
pub fn interceptor(mut req: Request<()>) -> Result<Request<()>, Status> {
    global::get_text_map_propagator(|prop| {
        prop.inject_context(
            &tracing::Span::current().context(),
            &mut MetadataMap(req.metadata_mut()),
        )
    });
    Ok(req)
}

/// Method used with gRPC server to set Span ID propagated via HTTP Header to every gRPC route.
/// # Example:
/// ```ignore
/// Server::builder()
///     .trace_fn(tracing_utils::grpc::trace_fn)
///     .add_service(MyServiceServer::new())
///     .serve(addr.into())
///     .await
/// ```
pub fn trace_fn(req: &http::Request<()>) -> tracing::Span {
    let span = tracing::info_span!("gRPC request", ?req);
    let parent_cx =
        global::get_text_map_propagator(|prop| prop.extract(&HeaderExtractor(req.headers())));
    span.set_parent(parent_cx);
    span
}

struct MetadataMap<'a>(&'a mut tonic::metadata::MetadataMap);

impl<'a> Injector for MetadataMap<'a> {
    fn set(&mut self, key: &str, value: String) {
        if let Ok(key) = tonic::metadata::MetadataKey::from_bytes(key.as_bytes()) {
            if let Ok(val) = tonic::metadata::MetadataValue::from_str(&value) {
                self.0.insert(key, val);
            }
        }
    }
}
