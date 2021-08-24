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
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Relation {
    #[prost(string, required, tag = "1")]
    pub global_id: ::prost::alloc::string::String,
    #[prost(uint32, required, tag = "2")]
    pub local_id: u32,
    #[prost(message, required, tag = "3")]
    pub search_for: SearchFor,
    #[prost(message, repeated, tag = "4")]
    pub relations: ::prost::alloc::vec::Vec<Relation>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LogicOperator {
    #[prost(enumeration = "logic_operator::Operator", required, tag = "1")]
    pub operator: i32,
}
/// Nested message and enum types in `LogicOperator`.
pub mod logic_operator {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Operator {
        And = 0,
        Or = 1,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SearchFor {
    #[prost(enumeration = "search_for::Direction", required, tag = "1")]
    pub search_for: i32,
}
/// Nested message and enum types in `SearchFor`.
pub mod search_for {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Direction {
        Parents = 0,
        Children = 1,
    }
}
