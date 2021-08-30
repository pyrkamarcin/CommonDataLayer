# Query Router
The REST interface over the CDL, used for retrieving data from any repository.

## Running
To run the **query-router** requires the [Schema Registry][schema-registry] to be running and
the [Query Services][query-service] or the [Timeseries Query Services][query-service-ts] connected to their respective
repositories.

Configuration is available [here][qr-configuration]

_Note: Currently, the cache is valid forever: changing a schema's **query-service** address will not update in the **
query-router**._

## Functionality
REST API specification is available in [OpenAPI 3.0 spec][api-spec].

Currently, the **query-router** can:

- handle querying data by ID from document repositories,
- query range of data by ID from time series repositories,
- query data from repositories by SCHEMA_ID.

Rough sketch of working process is available in QR [documentation][query-router-arch-doc]

[qr-configuration]: ../../docs/configuration/query-router.md


[query-router-arch-doc]: ../../docs/architecture/query_router.md

[schema-registry]: ../schema-registry/README.md

[query-service]: ../query-service

[query-service-ts]: ../query-service-ts

[api-spec]: ./api.yml
