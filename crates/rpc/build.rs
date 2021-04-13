fn main() -> std::io::Result<()> {
    tonic_build::configure().compile(
        &[
            "proto/query_service.proto",
            "proto/query_service_ts.proto",
            "proto/schema_registry.proto",
            "proto/edge_registry.proto",
            "proto/object_builder.proto",
            "proto/materializer.proto",
            "proto/common.proto",
            "proto/generic.proto",
        ],
        &["proto/"],
    )
}
