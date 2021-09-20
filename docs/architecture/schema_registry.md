# Schema Registry

### Technical Description
The Schema Registry (`SR` for short) is responsible for storing configuration about the data types handled by CDL. It is a persistent graph database, that can be queried via gRPC (other means of interaction are in progress). Currently, user interaction is performed with the [GraphQL][GraphQL] interface. Replication across multiple instances of the Schema Registry is supported.

Interacts with:
- nothing on its own

Is used by:
- Data Router
- Query Router
- General Materializer
- On Demand Materializer
- Object Builder
- Partial Update Engine

Query methods:
- GRPC
- GraphQL interface

Communication methods (supported repositories):
- Kafka (with other schema-registry instances)

[GraphQL]: api.md
