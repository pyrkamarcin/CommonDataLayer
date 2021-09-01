# Command Services
Services that translate messages received from the [Data Router][data-router] into their respective database's format.
Currently, only one Command Service implementation exists and is built in a such way that it can support multiple
databases (one at a time).

### Technical Description
The Command-Service (commonly referred also as `CS`, or `CSPG` - indicating postgres instance), interfaces storage
repositories with the CDL ecosystem.

Interacts with:

- Data Router (optional, either)
- Message Queue (optional, either)
- Supported Repository (one of)

Ingest methods:

- Kafka
- RabbitMq
- GRPC (currently either only one instance without kubernetes)

Egest methods (supported repositories):

- Postgresql (tested on 12, should support anything >=9, advised 13)
- VictoriaMetrics
- Druid
- Sleight (CDL's document storage)
- Troika (CDL's binary data repo)
- ... or anything with matching GRPC :)

[data-router]: data_router.md
