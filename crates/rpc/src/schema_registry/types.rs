use super::schema_type;

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
