#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RowDefinition {
    #[prost(message, repeated, tag = "1")]
    pub objects: ::prost::alloc::vec::Vec<Object>,
    #[prost(map = "string, string", tag = "2")]
    pub fields:
        ::std::collections::HashMap<::prost::alloc::string::String, ::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Object {
    #[prost(string, required, tag = "1")]
    pub object_id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub schema_id: ::prost::alloc::string::String,
}
