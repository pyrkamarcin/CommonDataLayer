# Edge registry

### Technical Description

Registry is responsible for storage of relations between schemas and objects.

### Communication

There are two methods of communicating with `ER` - gRPC and MessageQueue (RabbitMQ and Kafka are supported in this place).

#### gRPC communication

GRPC communication allows accessing whole feature set of `ER` and is required for querying.
List of available commands can be found in registry's [proto file](https://github.com/epiphany-platform/CommonDataLayer/tree/develop/crates/rpc/proto).

#### Message queue communication

MQ currently serves as  an alternative means of ingestion for object relation data (called `edge` within registry).
Messages must follow JSON Schema:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "array",
  "items": [
    {
      "type": "object",
      "properties": {
        "relation_id": {
          "type": "string",
          "pattern": "[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}"
        },
        "parent_object_id": {
          "type": "string",
          "pattern": "[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}"
        },
        "child_object_ids": {
          "type": "array",
          "items": [
            {
              "type": "string",
              "pattern": "[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}"
            }
          ]
        }
      },
      "required": [
        "relation_id",
        "parent_object_id",
        "child_object_ids"
      ]
    }
  ]
}
```

eg.:

```json
[
    {
      "relation_id": "4d987502-8800-11eb-b5cb-0242ac130003",
      "parent_object_id": "79bbc2d5-92a6-43ad-b182-d6b9dd49184c",
      "child_object_ids": [
        "627f84c7-d9f0-4665-b54d-2fcb5422ce02", 
        "627f84c7-d9f0-4665-b54d-2fcb5422ce03"
      ]
    }
]
```

Each entry in top level array represents one-to-many relation within `relation_id`. 
Such `relation_id` should be added beforehand, via gRPC api, between objects schemas.
`ER` at this time does not validate correctness of inserted data, so it's up to user to ensure that `edges` and `relations` are configured properly.

## Endpoints
### Resolve tree
The endpoint's purpose is to provide information on complex relationships on the object level. In SQL analogy resolve tree is similar to the `inner join` clause with filtering(based on `object_id`, `schema_id` values). You can 'join' multiple relations at once with a single resolve tree query.

Currently, resolve tree inner implementation generates complex SQL statements based on its arguments. This allows us to utilize complex filtering(support for `AND`, `OR` operators) without the need to handle the filtering ourselves.

For each base object(object_id from queried base schema) we return base object_id followed by related object_ids(order defined in a query) after filtering applied.

In the future we may consider adding support for `outer joins`, however, for now only `inner join` is supported as this is the only join strategy used internally by CDL for the time being.
