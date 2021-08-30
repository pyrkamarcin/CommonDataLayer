# Schema Registry

### Technical Description

The Schema Registry (`SR` for short) is responsible for storing configuration about the data types handled by CDL. It is a persistent graph database, that can be queried via gRPC (other means of interaction are in progress). Currently there is no GUI nor TUI; user interaction is currently performed with the [CDL-CLI][CDL-CLI]. Replication across multiple instances of the Schema Registry is supported.

Interacts with:
- nothing on its own

Is used by:
- Data Router
- Query Router
- cdl-cli

Query methods:
- gRPC (clients may use cdl-cli CLI application)

Communication methods (supported repositories):
- Kafka (with other schema-registry instances)

[CDL-CLI]: cli.md
