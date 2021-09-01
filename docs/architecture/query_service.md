# Query Service
Each Query Service serves a common set of queries, and translates those into their respective database's query language. Two query-services are present: one for time series type databases, and one for documents.

### Technical Description
The query service (`QS` or for example for PostgreSQL its `QSPG`), is responsible for querying data from specific repository. It offers two paths that can be accessed: First path depends on type of repo

### Communication
Communication to query service is done through [GRPC][grpc] based on two [endpoints][proto] of querying for data by `SCHEMA_ID` or multiple `OBJECT_ID`s. Query service communicates with multiple databases such as PostgreSQL, Apache Druid, VictoriaMetrics. Query service also communicates with [schema registry][schema-registry].

Interacts with:

- Druid
- PostgreSQL
- VictoriaMetrics (accidentally also Prometheus)
- .. any similar GRPC compatible repo

Query methods:
- GRPC (req-response)

Communication protocols:
- database specific

[GRPC]: https://grpc.io/docs/what-is-grpc/introduction/
[schema-registry]: schema_registry.md
[proto]: https://github.com/epiphany-platform/CommonDataLayer/tree/develop/crates/rpc/proto
