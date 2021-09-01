# Tutorial: How to materialize data in CDL
Tools used:

* GNU/Linux system
* `docker` and `docker-compose` - for running dependencies
* [`horust`](https://github.com/FedericoPonzi/Horust) for running CDL locally
* any GRPC client (either custom-made or generic like [BloomRPC](https://github.com/uw-labs/bloomrpc)) - for
  communication with schema registry.
* `python3` - for pushing data to kafka topic
* `psql` - for checking materialized data in Postgres
* web browser - for checking Jaeger traces & GraphQL interactive API.

# Preparing environment
The easiest way to set up an environment is to use [Common Data Layer deployment repository](https://github.com/epiphany-platform/CommonDataLayer-deployment).

You can use one of the examples provided there to run everything with one command.

## Setup
To materialize view, one needs proper setup.

Firstly, the system needs schema which informs where to store data (for example in document storage like Postgres). Secondly, it needs view definition which informs what fields are necessary and from which schema it should take it. It also defines where to put materialized data.

During this tutorial, we are going to introduce several ways of inserting and mutating state of CDL. One of them (GraphQL) is designed only for management and test purposes. However, it is the easiest way to quickly manually test the materialization pipeline.

### Manual setup
The most common and production-like way to set up is by sending requests to the schema registry.

#### Adding new schema

##### - GRPC API
To add new schema, we need to load schema_registry.proto to our client. In this tutorial, we are going to use BloomRPC.
All proto files are stored in `crates/rpc/proto` directory.
In the proto file, we can see that message `NewSchema` uses `bytes` as a definition. In BloomRPC it means we need to encode our JSON definition in base 64:

Schema definition:

```json
{
  "name": "string"
}
```

Encoded:

```base64
ewogICAgIm5hbWUiOiAic3RyaW5nIgp9
```

Usually, schema registry API is available at [http://localhost:50101](http://localhost:50101).

RPC request (`schema_registry.SchemaRegistry.AddSchema`):

```json
{
  "metadata": {
    "name": "tutorial-schema",
    "insert_destination": "cdl.document.1.data",
    "query_address": "http://localhost:50201",
    "schema_type": 2
  },
  "definition": "ewogICAgIm5hbWUiOiAic3RyaW5nIgp9"
}
```

RPC response:

```json
{
  "id": "22c8ac58-155e-4643-ab44-42e96dbb88c7"
}
```

Let's save this UUID for later. It is `schema_id`.

##### - GraphQL API (management and test purposes only)
Instead of using GRPC, we can also leverage GraphQL Gateway API to manage all schemas: An easy-to-use interface is available at [http://localhost:50106/graphiql](http://localhost:50106/graphiql)

Mutation request:

```graphql
mutation addSchema {
    addSchema(new: {
        insertDestination: "cdl.document.1.data",
        name: "tutorial-schema",
        queryAddress: "http://localhost:50201",
        type: DOCUMENT_STORAGE,
        definition: {
            name: "string"
        }
    }) { id }
}
```

As you can see, it does not require encoding definition, and it can return all schema metadata. Therefore, we filter it to retrieve only `schema_id`.

#### Adding new view
Next, we need to add a new view definition.

##### - GRPC API
GRPC request (`schema_registry.SchemaRegistry.AddViewToSchema`):

```json
{
  "schema_id": "22c8ac58-155e-4643-ab44-42e96dbb88c7",
  "name": "tutorial-view",
  "materializer_address": "http://localhost:50203",
  "materializer_options": "{\"table\": \"MATERIALIZED_VIEW\"}",
  "fields": {
    "worker_name": "{ \"field_name\": \"name\" }"
  }
}
```

As you can see, this time `materializer_options` are not using `bytes` format but are encoded in the string.
See [Issue #442](https://github.com/epiphany-platform/CommonDataLayer/issues/442).

GRPC response:

```json
{
  "id": "ddfc1f23-7b13-4d8e-b8ab-1def8eb30a4e"
}
```

This is the `view_id`.

##### - GraphQL API (management and test purposes only)
Mutation request:

```graphql
mutation addView{
    addView(schemaId: "22c8ac58-155e-4643-ab44-42e96dbb88c7", newView: {
        name: "tutorial-view",
        materializerAddress: "http://localhost:50203",
        materializerOptions: {
            table: "MATERIALIZED_VIEW"
        },
        fields: {
            worker_name: {
                field_name: "name"
            }
        },
        relations: []
    }) {
        id
    }
}
```

### Loading initial schema
While manual setup is fine for a one-time test, it quickly becomes mundane work. To mitigate this problem, we created a solution to pre-populate the schema registry. In fact, [Common Data Layer deployment repository](Common Data Layer deployment repository) already contains a file which describes what views and schemas should be inserted on startup (`bare/setup/schema-registry/initial-schema.kafka.json`)

### Inserting data
To materialize data first we need to insert it in the CDL.

##### - Python script
For that purpose, we can write very simple Python script:

```python
from kafka import KafkaProducer
from kafka.errors import KafkaError

producer = KafkaProducer(bootstrap_servers=['localhost:9092'])

with open('data.json', rb) as binary_file:
    data = binary_file.read()
    future = producer.send('cdl.data.input', data)

    try:
        record_metadata = future.get(timeout=10)
    except KafkaError:
        log.exception()
        pass

    print(record_metadata.topic)
    print(record_metadata.partition)
    print(record_metadata.offset)
```

`data.json` should look like:

```json
[
  {
    "objectId": "dc8cc976-412b-11eb-8000-100000000000",
    "schemaId": "22c8ac58-155e-4643-ab44-42e96dbb88c7",
    "data": {
      "name": "John"
    }
  },
  {
    "objectId": "dc8cc976-412b-11eb-8000-000000000000",
    "schemaId": "22c8ac58-155e-4643-ab44-42e96dbb88c7",
    "payload": {
      "name": "Alice"
    }
  }
]
```

##### - GraphQL API (management and test purposes only)
GraphQL's request:

```graphql
mutation insertBatch {
    insertBatch(
        messages: [
            {
                objectId: "dc8cc976-412b-11eb-8000-100000000000",
                schemaId: "22c8ac58-155e-4643-ab44-42e96dbb88c7",
                payload: { name: "John" },
                version: "1.0.0"
            }
            {
                objectId: "dc8cc976-412b-11eb-8000-000000000000",
                schemaId: "22c8ac58-155e-4643-ab44-42e96dbb88c7",
                payload: { name: "Alice" },
                version: "1.0.0"
            }
        ]
    )
}
```

### Querying materialized data
After a second, we should be able to see our materialized view in Postgres.

```sh
psql -U postgres --password -h localhost
```

The default password for local dev Postgres is `1234`, but shhh, don't tell anyone ;-)

```
postgres=# select * from cdl.materialized_view;
              object_id               | worker_name
--------------------------------------+-------------
 dc8cc976-412b-11eb-8000-100000000000 | "John"
 dc8cc976-412b-11eb-8000-000000000000 | "Alice"
(2 rows)
```

### On-Demand materialization
Instead of waiting seconds for on-the-fly materialization, we can also demand materialized view via GRPC call.

#### GRPC API
GRPC request to [http://localhost:50108/](http://localhost:50108/) (`materializer_ondemand.OnDemandMaterializer.Materialize`):

```json
{
  "view_id": "ddfc1f23-7b13-4d8e-b8ab-1def8eb30a4e",
  "schemas": {
    "22c8ac58-155e-4643-ab44-42e96dbb88c7": {
      "object_ids": [
        "dc8cc976-412b-11eb-8000-100000000000",
        "dc8cc976-412b-11eb-8000-000000000000"
      ]
    }
  }
}
```

For now, user needs to use filter and enlist in the request all object IDs. There is, however, [an issue which should mitigate this problem](https://github.com/epiphany-platform/CommonDataLayer/issues/429) very soon. This call returns the stream of rows, instead of collection. Thanks to that, both object builder, on-demand Materializer and client code don't have to allocate enormous amount of memory when handling bigger tables. It also means client code can start processing data faster.

BloomRPC returns messages in separate tabs:

Stream 1:

```json
{
  "fields": {
    "worker_name": "\"John\""
  },
  "object_id": "dc8cc976-412b-11eb-8000-100000000000"
}
```

Stream 2:

```json
{
  "fields": {
    "worker_name": "\"Alice\""
  },
  "object_id": "dc8cc976-412b-11eb-8000-000000000000"
}
```

#### GraphQL API (management and test purposes only)
GraphQL's request:

```graphql
query onDemandView {
    onDemandView(
        request: {
            viewId: "ddfc1f23-7b13-4d8e-b8ab-1def8eb30a4e"
            schemas: [
                {
                    id: "22c8ac58-155e-4643-ab44-42e96dbb88c7"
                    objectIds: [
                        "dc8cc976-412b-11eb-8000-100000000000",
                        "dc8cc976-412b-11eb-8000-000000000000"
                    ]
                }
            ]
        }
    ) {
        id
        rows {
            fields
            objectId
        }
    }
}
```

Unfortunately, GraphQL does not support streaming, which means all rows are collected to the array before sending it to the client. Please use it wisely and carefully (on smaller sets of data).

### Appendix: Checking traces in Jaeger
For troubleshooting or when you are curious, how CDL works, we recommend using Jaeger telemetry sink, which by default is available at [http://localhost:16686/search](http://localhost:16686/search).


[Common Data Layer deployment repository]: https://github.com/epiphany-platform/CommonDataLayer-deployment
