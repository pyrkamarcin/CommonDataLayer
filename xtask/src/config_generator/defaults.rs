macro_rules! default_service {
    (
        $(
            [@ $port:expr, $($kw:ident $var_name:ident),*],
        )*
    ) => {
        $(
            $(default_var!($port, $kw $var_name);)*
        )*
    };
}

macro_rules! default_var {
    ($port:expr,port $var_name:ident) => {
        pub const $var_name: u16 = $port;
    };
    ($port:expr,host $var_name:ident) => {
        pub const $var_name: &str = concat!("http://localhost:", stringify!($port));
    };
    ($port:expr,serve $var_name:ident) => {
        pub const $var_name: &str = concat!("0.0.0.0:", stringify!($port));
    };
}

// external service defaults
pub const DEFAULT_KAFKA_BROKERS: &str = "localhost:9092";
pub const DEFAULT_VICTORIA_METRICS_HOST: &str = "http://localhost:8428";

// default topics / queues
pub const DEFAULT_DATA_ROUTER_INGEST_SOURCE: &str = "cdl.data.input";
pub const DEFAULT_CDL_NOTIFICATION_CHANNEL: &str = "cdl.reports";
pub const DEFAULT_EDGE_REGISTRY_NOTIFICATION_SOURCE: &str = "cdl.edge.input";
pub const DEFAULT_PUE_EGEST_TOPIC: &str = "cdl.materialization";

default_service!(
    // 501xx port space: client - cdl communication
    [
        @ 50102,
        serve DEFAULT_DATA_ROUTER_LISTEN_URL
    ],
    [
        @ 50103,
        port DEFAULT_QUERY_ROUTER_PORT,
        host DEFAULT_QUERY_ROUTER_HOST
    ],
    [
        @ 50106,
        port DEFAULT_API_PORT
    ],
    // 502xx port space: repository in&egestion
    [
        @ 50201,
        serve DEFAULT_COMMAND_SERVICE_LISTEN_URL
    ],
    [
        @ 50202,
        port DEFAULT_QUERY_SERVICE_PORT
    ],
    [
        @ 50212,
        port DEFAULT_QUERY_SERVICE_TS_PORT
    ],
    // 503xx port space: materialization
    [
        @ 50301,
        port DEFAULT_GENERAL_MATERIALIZER_PORT
    ],
    [
        @ 50302,
        port DEFAULT_ON_DEMAND_MATERIALIZER_PORT,
        host DEFAULT_ON_DEMAND_MATERIALIZER_HOST
    ],
    [
        @ 50303,
        port DEFAULT_OBJECT_BUILDER_PORT,
        host DEFAULT_OBJECT_BUILDER_HOST
    ],
    // 504xx port space: configuration
    [
        @ 50401,
        port DEFAULT_SCHEMA_REGISTRY_PORT,
        host DEFAULT_SCHEMA_REGISTRY_HOST
    ],
    [
        @ 50402,
        port DEFAULT_EDGE_REGISTRY_PORT,
        host DEFAULT_EDGE_REGISTRY_HOST
    ],
);
