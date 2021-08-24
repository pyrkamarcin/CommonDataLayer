# Front Matter

```
    Title           : Edge registry - Resolving relation trees
    Author(s)       : kononnable
    Team            : CommonDataLayer
    Reviewer        : CommonDataLayer
    Created         : 2021-07-07
    Last updated    : 2021-08-04
    Category        : Feature
    CDL Feature ID  : CDLF-0001F-00
```

## Glossary
- Relation - link between objects of two schemas. Relation represents information on a type level, it doesn't connect two specific objects, it simply states that objects of the following schemas may be related to each other
- Relation graph - information about multiple related objects, single relation graph can present data from multiple relations, currently presented in form of a tree, proposal changes representation form to table
- TreeQuery - part of Edge registry API responsible for generating relation graphs for provided recipe and filtering criteria
- RDBMS - relational database management system

## Motivation
Currently, Edge Registry supports a list of relations that together creates non-cyclic graphs. However, using formats as they are currently defined have severe limitations as there is no way to differentiate which direction the links in the graph are pointing to (i.e. parent or a child objects). In addition, there is no way to specify a filter composed of multiple components, bound together with usage of logical operators like `or` or `and`. Complex filters are a requirement for proper materialization support.

This document's purpose is to pass on the way behavior can be handled.

Struct names are treated as implementation detail which may be changed during implementation.

## Current State

### TreeQuery Request
```rust
struct TreeQuery {
    relation_id: Uuid,
    relations: Vec<TreeQuery>,
    filter_ids: Vec<Uuid>,
}
```

### TreeQuery Response
```rust
struct RelationTree {
    objects: Vec<TreeObject>,
}

struct TreeObject {
    object_id: Uuid,
    relation_id: Uuid,
    relation: SchemaRelation,
    children: Vec<Uuid>,
    subtrees: Vec<RelationTree>,
}

struct SchemaRelation {
    parent_schema_id: Uuid,
    child_schema_id: Uuid,
}
```

## Proposed Formats

### TreeQuery Request
```rust
type Identifier = Option<NonZeroU8>; // id from TreeQuery->Relation, None for base_relation
```
The team decided that the Identifier field presented above will use this format instead of plain u8, however, this change should affect only Rust code, protofile doesn't have to treat this field as an enum.

```rust
struct Relation {
    relation_id: UUID,
    identifier: NonZeroU8, // Unique in request
    side: RelationSide,
    relations: [Relation],
}

struct TreeQuery{
    base_relation: Uuid,
    filters: Option<Filter>,
    relations: [Relation],
}

enum Filter {
    SimpleFilter,
    ComplexFilter,
}

struct ComplexFilter {
    operator: LogicOperator,
    operands: [Filter],
}

// To be extended later
enum LogicOperator {
    And,
    Or,
}

enum SimpleFilter {
    InParentObjIds {
        relation: Identifier,
        ids: Vec<Uuid>
    },
    InChildObjIds{
        relation: Identifier,
        ids: Vec<Uuid>
    }
}

enum RelationSide {
    Parents,
    Children,
}
```

### TreeQuery Response
```rust
struct TreeQueryResponse {
    resp: Vec<Uuid> // number of rows * (2 + TreeQuery.relations.len())
}
```

The new response format will provide UUIDs in the following order:
1. relation's parent's ID
2. relation's child's ID
3. array of relation's children's IDs

### Summary
Change explanation related to previous format:
- previously Object Builder was using flat form, its first step was to flatten old response format. This is no longer the case, as the response will be in tree format.
- in old format, information about relations was sent multiple times, containing one edge at the time. Currently, we will send the whole tree as a response, including relations. More info in [Getting relation information](#getting_relation)
- It might seem that this format introduces duplication(some IDs will be sent multiple times), however that shouldn't be a problem. This is the ideal case for usage of lossless algorithms on protocol level(if needed). Similar solutions(table results) are widely used with RDBMS and I don't recall any performance issues about sending the data.

### Processing TreeQuery Request
- Resolving relation tree should be handled by using `inner join` strategy. This may be configurable in the future.
- Filtering based on object_ids should be performed.
- It might be a good idea to let SQL engine to do almost all the work for us and just generate SQL query and collect the results. Efficient joining and filtering data is one of the main goals of RDBMS, and it doesn't require as much network traffic as doing it on our own. This is just a proposal - implementor is free to choose which way this should be handled.

### <a name="getting_relation"></a>Getting Relation Information
```patch
service EdgeRegistry {
...
+  rpc GetRelation(RelationQuery) returns (RelationList);
-  rpc GetRelation(RelationQuery) returns (RelationResponse);
...
-  rpc ListRelations(Empty) returns (RelationList);
}
...
message RelationQuery {
-  required string relation_id = 1;
+  repeated string relation_id = 1;
-  required string parent_schema_id = 2;
}
```
`Parent schema ID` is not required for fetching relation information. I suggest removing it and allowing to query multiple relations in one request. This will also make `ListRelations` obsolete, as the same thing should be possible to achieve by sending empty `RelationQuery`.

Extracting relation metadata from TreeQuery shouldn't cause any performance problems, as those two requests can(should) run in parallel by OB.

## Related RFCs
-  [0006 Edge registry](./0006_Edge_registry_01.md)
-  [0007 Materialized views](./0007_Materialized_views_01.md)
