# GraphQL API
A server which provides `/graphql` and `/graphiql` routes for CDL management. It is a self-describing, interactive, and easy-to-use way to manage your instance.

To learn about the GraphQL protocol, we recommend visiting the [official tutorial](https://graphql.org/learn/).

## Usage
The GraphQL API is based on an HTTP server that listens on a port set by the configuration key `input_port`.

Usually, it is set to `50106`. Therefore, for example, by using the [Horust bare metal deployment](https://github.com/epiphany-platform/CommonDataLayer-deployment), you can access the API and the playground at, respectively:
* [http://localhost:50106/graphql](http://localhost:50106/graphql)
* [http://localhost:50106/graphiql](http://localhost:50106/graphiql)

## GraphQL Route - /graphql
This route is a raw route that can be used, for example, in the frontend to communicate with the CDL. Usually, the frontend needs a client library like [`@apollo/react`](https://www.apollographql.com/docs/react/) that can connect to the API.

## Interactive GraphQL Route - /graphiql
This is the easiest way to play with the CDL quickly. It is a batteries-included GraphQL IDE, where you can write queries and see responses in real-time. It has built-in documentation, history, auto-completion, and it preserves its state (like past queries) in local storage.

## API
All examples below can be executed on the Interactive GraphQL Route (just copy and paste it to the editor) or as a base for a call from GraphQL library. For example, in JavaScript + React:

```js
const GET_SCHEMAS = gql`...INSERT_HERE_QUERY...`;

function Component() {
    const {loading, data} = useQuery(GET_SCHEMAS);
...
}

```

For more information how to connect and execute the query, we recommend visiting [Official website](https://graphql.org/learn/) and a client library like [`@apollo/react`](https://www.apollographql.com/docs/react/).

### Querying
Schema Registry can be deployed with an automatically initialized state, in which a table for schemas is set up on the first startup; If that's the case, you can start making queries right away, like:

``` graphql
query GetSchemas {
    schemas {
      id,
      definitions {
        version,
        definition
      },
      views {
        expression
      }
    }
}
```

### Mutating
The GraphQL API can also change the state of the CDL, including inserting data or changing configuration (like schema definitions).

To edit the configuration of the CDL, you can write a mutation query:

```graphql
mutation AddView {
    addView(
        schemaId: "6eb25fd5-186c-46b1-a149-947825362bee"
        newView: {
            name: "Friendly"
            materializerAddress: "http://localhost:50203",
            materializerOptions: {table: "MATERIALIZED_VIEW"},
            fields: {
                worker_name: { simple: { field_name: "name", field_type: "string" } }
            }
            relations: [
                {
                    globalId: "97b4317e-bdf9-11eb-b579-0242ac120003"
                    localId: 1,
                    searchFor: "CHILDREN"
                    relations: []
                }
            ]
        }
    ) {
        id
    }
}
```

### Subscriptions
If you ever need to listen to status notifications in real-time, the GraphQL API supports subscriptions by using WebSockets under the hood.

##### Example

```graphql
subscription notifyMe {
    reports {
        application,
        description,
        outputPlugin,
    }
}
```

