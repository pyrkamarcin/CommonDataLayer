# Front Matter

```
    Title           : Simplify Schema Definitions
    Author(s)       : Samuel Mohr
    Team            : CommonDataLayer
    Reviewer        : CommonDataLayer
    Created         : 2021-07-06
    Last updated    : 2021-07-06
    Category        : Feature
    CDL Feature ID  : CDLF-00020-00
```

## Glossary

### Terminology
* CDL - Common Data Layer
* SR - Schema Registry, a CDL component responsible for keeping information about the type of object conveyed inside the message.
* User - user of the CDL.
* Schema Definition - a schematic representing the expected format for data belonging to a given Schema.

## Formats
Schema:
```
{
    id: UUID,
    name: String,
    definition: Map<String, ObjectFieldDefinition>, // definitions can only be non-null objects
    ... // other configuration metadata
}
```

ScalarType:
```
Bool | String | Integer | Decimal | Any // Any represents arbitrary JSON data
```

ObjectFieldDefinition:
```
{
   type: ScalarType,
   optional: Bool,
}
```

SchemaDefinition:
```
Scalar {
    type: ScalarType,
    optional: Bool,
} |
Object {
    fields: Map<String, ObjectFieldDefinition>,
} |
Array {
    item_type: ChildType,
    optional: Bool,
}
```

## Introduction

### Background
Schemas, as defined in the SR, are a means to represent discrete types of data to be stored in the CDL. Beyond metadata such as the name of the schema and addresses for repository storage and retrieval, they also store definitions describing the format of their respective data types. Though the data is not currently being validated based on its type, the type definition is useful for understanding the data format. In addition, since materialization stores values based on views, which use types from schemas, we can use the field types of schema definitions to determine what types a view comprises. This lets us more efficiently store materialized data.

Currently, though not used, each schema has a definition, defined using a JSON Schema. [JSON Schema][JSON Schema] is a protocol for constraining data stored as JSON. On top of tradition typing of fields, it also can perform complex checks, such as logical operations with respect to field existence and numerical constraints on values. It can also define relationships to other schemas, which is a behavioral requirement for the CDL's schemas. These checks are generally valuable, and led to us choosing it as a means for defining schema data types some months ago. However, the inherent complexity of JSON Schema prevents us from being able to simply determine what type a field is in a definition. So long as we maintain JSON Schema as the solution for user-defined data definitions, it will remain infeasible to determine types of individual fields in views for more efficient materialization (spatially and temporally).

### Assumptions
There is not currently a need to provide mutative access to schema definitions, so the re-implementation of schema definitions does not need to work with that in mind, though a solution robust to that potential future requirement may be preferable.

## Proposed Solution
There does not exist, in the space of the Rust ecosystem, an existing solution for us to use. Therefore, this RFC proposes a novel-yet-simple system for defining schema definitions allowing for definition composition and representation of all valid JSON data. The proposed new form of schema definitions will be referred to as Schema Definitions, or SD's for the rest of this document.

Currently, JSON schemas are stored as metadata of schemas in a one-to-one relationship, disallowing the reuse of schema definitions between schemas. In the new solution, this will change where each definition is stored with a unique ID (a UUID will suffice) and the provided definition. Each schema will be required to hold a definition ID. This will allow for easier relationship definition between SD's, and potentially allowing for "mutation" of SD's without actually removing the old definition from the SR. These aspects will be explained further on in this proposal.

In an effort to maintain a similar structure to JSON data and not added unnecessary complexity, SD's will be represented as sum types. Beyond a variant for each of the scalars available in JSON (`Boolean`, `String`, `Integer`, and `Decimal`), the other two types of JSON data are `Array`s and `Object`s. `Array`s will refer to a single child definition, whether that be a scalar type or a different schema's definition. `Object`s will refer to a list of string-named fields, where the definition of each field is either a scalar or another Schema's definition. `Array` or `Object` child definitions can all be optional, which allows items to either be `Null` or missing, in the case of fields in `Object`s.

In the case that some JSON data is too complicated to be represented by the proposed system, or a user would like to opt out of the optimizations provided by properly-typed schemas, the `Any` scalar variant will accept any valid JSON data.

### Use in Materialization
With this system, all fields in a schema definition have deterministic types. Given that, any view that references fields from an `Object` schema definition can know the field's type. Either the type is a potentially nullable scalar, which can be represented by a SQL type, or it is a more complicated type (or an `Any` scalar), and it falls back to a JSON type. That way, all materialized data can have its fields stored in its respective types.

### Schema Composition
There is a desire from CDL users to define data that is related in definition to other schemas, whether that be via inheritance or composition. As composition is generally less problematic and simpler to implement than inheritance, it could be implemented by slightly modifying the above specification. However, as composition/inheritance is not needed at the moment and is a complex enough addition that it will require another RFC to properly describe, it will be left out of the initial version here described.

### SQL Type Mapping
For the most part, materialization will store calculated values in SQL-style databases. For most scalar types, there is a well-defined mapping that describes what type to use for any given database (e.g., booleans and strings). There are, on the other hand, types that do not have an agreed upon representation.

Proposed is the suggestion to store for each database materialized data is stored in a list of type mappings. This would be in the config service (really, wherever the materialization configuration data is stored).

## Further Considerations

### Impact on Other Teams/Clients
Though validation is not currently being done on incoming data and therefore schema definitions don't matter much at the moment, this will change not only all currently stored schema definitions but also all future schema definition insertions. There will potentially need to be a one-time migration if anyone wants to keep their schema definition. Likely, the volume of such request for conversion will be low enough to be doable manually.

However, we plan to explicitly document to our end users that we do not plan to run the migrations for them, as we do not currently have any users relying on this feature and do not want to promise capacity we cannot fill.

### Scalability
No impact. This simply changes the way that read-only Schemas have some metadata formatted.

### Testing
This feature should undergo reasonable amounts of testing. Testing should include:

* Unit Tests:
    * Determination of view types from schema definition, simple and complex
    * Rendering of types to users, including schema composition
* Green Path Test


[JSON Schema]: https://json-schema.org/
