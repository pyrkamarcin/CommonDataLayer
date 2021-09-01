
# Query Router

### Technical Description
The Query Router (`QR`), is responsible for forwarding requests to specific query services. In CDL messages can be stored in any available repository, data router acts as a single entry point to multi-repo system and query router allows that data to be fetched easily. Query Router first queries SR, then basing on received config, finds out specific QS that, hopefully, should be able to respond to a specific query. Logic of that process is based on repo_type and query-service address stored with schema itself.

### Communication
Interacts with:

- Query Service
- Schema Registry

Query methods:

- REST (request-response)

Communication protocols:

- gRPC with query-services (request-response)
- gRPC with schema-registry (request-response)

## Running
To run the **query-router** requires the [Schema Registry][schema-registry] to be running and the [Query Services][query-service] connected to its respective repository.

_Note: Currently, the cache is valid forever: changing a schema's **query-service** address will not update in the **
query-router**._

## Functionality
REST API specification is available in [OpenAPI 3.0 spec][api-spec].

Currently, the **query-router** can:

- handle querying data by ID from document repositories,
- query range of data by ID from time series repositories,
- query data from repositories by SCHEMA_ID.

Rough sketch of working process:

```plantuml
@startuml
skinparam backgroundColor #FEFEFE
skinparam transparent false

actor "User (REST)" as REST
participant "Query Router" as QR
database "Schema Registry" as SR
database "Query Service" as QS

REST -> QR: Query for data
QR -> SR: Query for schema_id
SR -> QR: Retrieve associated query_service, data_type
QR -> QS: Query for data
QS -> QR: Retrieve document
QR -> REST: Retrieve document
@enduml
```

[schema-registry]: schema_registry.md
[query-service]: query_service.md
[query-service-ts]: https://github.com/epiphany-platform/CommonDataLayer/tree/develop/crates/query-service-ts
[api-spec]: https://github.com/epiphany-platform/CommonDataLayer/blob/develop/crates/query-router/api.yml
