# Object builder

### Technical Description

Object builder responsibility is creating complex objects according to recipes - view definitions. 

Object Builder Loop:
- Wait for request to build a view 
- Fetch view definition from schema registry
- Fetch objects from repositories
- Perform filtering by custom fields, join operations 
- Send data to materializers component or to the requesting party

It is important to note that object builder output contains view id, change list received from partial update engine, and requested objects with information how they were created (each returned object contains ids of every object which was used for its creation). 

### Communication

There are two methods of communicating with `OB` - gRPC and MessageQueue (RabbitMQ and Kafka are supported in this place).

#### gRPC communication

gRPC communication allows to materialize view on demand. Materialized view is not saved in any database, but sent as a response via gRPC.

#### Message queue communication

MQ currently serves as a main method of ingestion for view that needs to be materialized in database.
Messages payload are just UUIDs of the view that needs to be updated/created. There is no JSON encoding.

eg.:

```
627f84c7-d9f0-4665-b54d-2fcb5422ce02
```

