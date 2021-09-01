use super::{scalar_type, schema_type};

rpc_enum! {
    SchemaType,
    schema_type::Type,
    schema_type,
    "schema type",
    "schema_type_enum",
    [
        DocumentStorage,
        Timeseries
    ]
}

rpc_enum! {
    ScalarType,
    scalar_type::Type,
    scalar_type,
    "scalar type",
    "scalar_type_enum",
    [
        Bool,
        String,
        Integer,
        Decimal,
        Any
    ]
}
