
# Front Matter

```
Title           : Materialization field types support
Category        : Feature
Author(s)       : Łukasz Biel
Team            : CommonDataLayer
Created         : 2021-16-08
Deadline        : 2021-21-08
CDL feature ID  : CDLF-00018-00
```

#### Related
* https://github.com/epiphany-platform/CommonDataLayer/issues/656

# Glossary
* TS - timeseries
* QSTS - `query-service` for timeseries
* QS - `query-service` for documents
* OB - object builder
* ES - elasticsearch
* PSQL - PostgreSQL
* SR - Schema Registry
* Blob - `any` data, eg. pdf report, ms office spreadsheet, jpeg image, that CDL shouldn't preprocess when storing it
* ADT - algebraic data type ([wiki][wiki-adt])
* ConfS - configuration service, central repository of CDL configuration, currently in rfc phase.

# Summary
This rfc aims to provide and standardize types in CDL schemas, views and materialization for documents.

# TLDR
Schemas support
```
number
string
boolean
object
```

Any other type used within JSON schema should cause an error when view materialization is requested.

Views support (names are arbitrary, implementor may choose suitable naming)
```
f64
i64
string
boolean
object - Field::Object
```

PSQL mapping is
```
f64 -> float8
i64 -> int8
string -> text
boolean -> bool
object -> each value resides in a column with `_` delimited json field path as a name
```

ES mapping is
```
f64 -> double
i64 -> long
string -> text
boolean -> boolean
object -> !supported by default
```

We require users to specify all types within the view, including deeply nested objects. This is supported by materialization.

We can later expand this type list with per database custom types (and type casting/mapping) but this requires schema language extension or parsing strings in JSON object.

SR emits a notification when a view is added, initiating materialization for it and provisioning the database.


# Configuration layer

## How is data stored?

### Documents
JSON objects. They are not indexed by any field and querying for only parts (fields or sub-documents)
requires fetching the whole JSON and then performing operations on it. 

> SIDENOTE:
> In practice PSQL can query for sub-documents (it's not as performant as querying on indexed fields). It's CDL that doesn't support this feature by default.

### Timeseries
Timeseries data is managed by databases, often requires special queries on QSTS side. User usually can query for a specified field. However, this depends on the database.

### Blob
Blobs are stored as is. You cannot perform any operations on it (except for fetch and store, of course).

> SIDENOTE:
> Other storage types are included for future reference. Currently, materialization is limited to Document Store.  
> If that was to change, new document would have to be created, exploring the integration.  
> From this point on we are going to focus on document storage only.

## JSON
JSON has fairly limited type system. We can read more about it in [JSON Schema spec][json-schema-basic-types].
CDL currently requires users to provide JSON Schema when declaring object's schema in SR.
> SIDENOTE:  
> While it's required in SR, it's not really "used". It is planned to be incorporated in validation, and may be useful with views.
> [rfc 0019][rfc-0019] talks about simpler format, as a replacement and for the sake of Materialization I will explore expanding it later.

| type name | description                                            |
|:----------|:-------------------------------------------------------|
| number    | any number of any precision                            |
| string    | Unicode string of characters of any length             |
| boolean   | "true" or "false" keyword                              |
| null      | a null value, or, to be precise, lack of thereof       |
| object    | a json key-value, unordered, map                       |
| array     | an ordered collection of records, can hold mixed types |

JSON Schema allows also to construct "one_of" ADTs. This is slightly controversial, as neither PSQL nor ES support that kind of construct for most of their types.
With one exception. Nullability is often described in json schema with `"one_of": any | null` and both databases house nullability feature.
This means that CDL may have to be able to recognize this pattern and act accordingly, setting up materialization output with nullability in mind.

# Materialization Storage Layer

## Postgresql
Before I start listing Postgres data types, there's one issue to mention - at this time we don't have specified
version of PSQL that we support. There are minor discrepancies in both behavior and (what is relevant here) type
system between 9.6 and 13.

```diff
21a22
> macaddr8        MAC (Media Access Control) address (EUI-64 format)
25a27
> pg_snapshot     user-level transaction ID snapshot
39c41
< txid_snapshot       user-level transaction ID snapshot
---
> txid_snapshot       user-level transaction ID snapshot (deprecated; see pg_snapshot)
```

In the diff excerpt we see that 13 introduces new types `macaddr8` and `pg_snapshot`, and deprecates `txid_snapshot`. I
have to mention also that `pg_snapshot` is not present in PSQL 12, but `macaddr8` is. This may prove an issue when dealing with non-basic types.

```
| name                                    | aliases            | description                                        |
|:----------------------------------------|:-------------------|:---------------------------------------------------|
| bigint                                  | int8               | signed eight-byte integer                          |
| bigserial                               | serial8            | autoincrementing eight-byte integer                |
| bit [ (n) ]                             |                    | fixed-length bit string                            |
| bit varying [ (n) ]                     | varbit [ (n) ]     | variable-length bit string                         |
| boolean                                 | bool               | logical Boolean (true/false)                       |
| box                                     |                    | rectangular box on a plane                         |
| bytea                                   |                    | binary data (“byte array”)                         |
| character [ (n) ]                       | char [ (n) ]       | fixed-length character string                      |
| character varying [ (n) ]               | varchar [ (n) ]    | variable-length character string                   |
| cidr                                    |                    | IPv4 or IPv6 network address                       |
| circle                                  |                    | circle on a plane                                  |
| date                                    |                    | calendar date (year, month, day)                   |
| double precision                        | float8             | double precision floating-point number (8 bytes)   |
| inet                                    |                    | IPv4 or IPv6 host address                          |
| integer                                 | int, int4          | signed four-byte integer                           |
| interval [ fields ] [ (p) ]             |                    | time span                                          |
| json                                    |                    | textual JSON data                                  |
| jsonb                                   |                    | binary JSON data, decomposed                       |
| line                                    |                    | infinite line on a plane                           |
| lseg                                    |                    | line segment on a plane                            |
| macaddr                                 |                    | MAC (Media Access Control) address                 |
| macaddr8                                |                    | MAC (Media Access Control) address (EUI-64 format) |
| money                                   |                    | currency amount                                    |
| numeric [ (p, s) ]                      | decimal [ (p, s) ] | exact numeric of selectable precision              |
| path                                    |                    | geometric path on a plane                          |
| pg_lsn                                  |                    | PostgreSQL Log Sequence Number                     |
| point                                   |                    | geometric point on a plane                         |
| polygon                                 |                    | closed geometric path on a plane                   |
| real                                    | float4             | single precision floating-point number (4 bytes)   |
| smallint                                | int2               | signed two-byte integer                            |
| smallserial                             | serial2            | autoincrementing two-byte integer                  |
| serial                                  | serial4            | autoincrementing four-byte integer                 |
| text                                    |                    | variable-length character string                   |
| time [ (p) ] [ without time zone ]      |                    | time of day (no time zone)                         |
| time [ (p) ] with time zone             | timetz             | time of day, including time zone                   |
| timestamp [ (p) ] [ without time zone ] |                    | date and time (no time zone)                       |
| timestamp [ (p) ] with time zone        | timestamptz        | date and time, including time zone                 |
| tsquery                                 |                    | text search query                                  |
| tsvector                                |                    | text search document                               |
| txid_snapshot                           |                    | user-level transaction ID snapshot                 |
| uuid                                    |                    | universally unique identifier                      |
| xml                                     |                    | XML data                                           |
```

Out of this list, we could use mapping `json::string` -> `psql::text` and `json::boolean` -> `psql::boolean`, and that's it.
Everything else has to be handled per case. Inference can't really work, as `json::number` may map to `real` or `int` or `bigint`.
Numbers also aren't capped anywhere in JSON's spec, which means that they may overflow.
Nullability is achieved via an additional flag on column: `NULL`.

Right now, we use JSON as type of all columns.

## Elasticsearch

| Elasticsearch type |
|:-------------------|
| null               |
| boolean            |
| byte               |
| short              |
| integer            |
| long               |
| double             |
| float              |
| half_float         |
| scaled_float       |
| keyword            |
| text               |
| binary             |
| date               |
| ip                 |
| object             |
| nested             |

This is a list of SQL mapped types, it's not fully exhaustive, more can be found on [ES documentation page][es-basic-data-types].
As we can see, again, there's greater granularity in ES than it is in JSON.

ES is able to deduce types based on insertions (first insertion to a given index generates type mapping), but it's questionable whether we'd like to make use of this feature.
Another main difference is that ES allows for storage of documents. While currently our PSQL implementation is flat (or to be more specific, it flattens everything to one table),
ES can store deeply nested objects. It also keeps an index on every field in a document.

# Proposal

## Type coverage
> DECISION:
> Type coverage is defined in TLDR section. We are omitting arrays and nullability for the time being. Objects are supported for both databases.

We should aim to, as an MVP, support a minimal subset of common types in both databases and add new ones if necessary in the later stages of development.
> SIDENOTE:  
> For types that are database specific (eg. psql::timestamp) we may add them in the form of an `optional feature`.

> SIDENOTE:
> JSON Schema is abstract here, however my choice of initial types is based on in and on Rust's `serde_json` ability to discern JSON types.
> To support more than just base, we will have to either extend JSON Schema or provide homebrew schema solution

I propose to use `String`, `bool`, `i64` and `f64` as base types for the first iteration, with addition of nullability and arrays of base types.
Objects, or complex arrays, are much more problematic, and we have to review whether we need them. I'll get to this in a moment.

PSQL mapping in this case:
```
`String` -> `psql::text`
`bool` -> `psql::bool`
`i64` -> `psql::int8`
`f64` -> `psql::float8`
`null` -> COLUMN IS NULLABLE # supported later as an extension
`array` -> type[] # supported later as an extension
```

ES mapping:
```
`String` -> `es::text`
`bool` -> `es::boolean`
`i64` -> `es::long`
`f64` -> `es::double`
`null` -> See note below # supported later as an extension
`array` -> See note below # supported later as an extension
```
> SIDENOTE:  
> Arrays and nullability are treated the same in ES, a field can contain 0 or more entries. [reference][es-arrays]

I propose to disallow mixed type arrays on schema level. This is not supported by either of our materialization databases.
If in future a client asks for those, we may consider `array[JSON]`.

### About `object` Type
ES requires that object schema is specified to each leaf. In Postgres, we can't really have leaves without introducing multiple tables per view.

There are few options on how to approach this topic:
#### ES requires "deep" types. PSQL stays with JSON.
It means that we have differing implementations between two databases, where one requires "type provider" to declare what is stored within the object and the other stores `JSON` as is.

#### ES Requires "Deep" Types. PSQL Uses Record Type.
We change PSQL to use a type that requires user to specify types in deeply nested objects, explicitly in view definition. This will mean ES and PSQL use the same strategy, but records are problematic.

#### ES Requires "Deep" Types. PSQL Uses Multiple Tables.
We can generate deeply nested relations using Postgres tables (duh!). It's not the easiest task, and that may require a separate document, but having this,
we'll be able to store the same data in both ES and PSQL.

Just for the sake of clarity, I mean:

table: main

| id        | field_a | obj_b         |
|:----------|:--------|:--------------|
| 0 PRIMARY | "abc"   | 1 FOREIGN KEY |

table: obj_b

| id        | field_b | field_c |
|:----------|:--------|:--------|
| 1 PRIMARY | "cde"   | 12.6f64 |

#### We Disallow for Deep Objects in Both
Everything has to be mapped to flat. This is probably impossible for clients to accept.

#### We Store Deep Objects as Strings in ES
This would mean an object is just a blob. We lose ES indexing ability, but we gain similar behavior to PSQL.

#### We Allow ES to Deduce Type on Its Own
This may be a bit error-prone and require extra testing, we would only provision PSQL with types and let ES do its type inference.

#### CDL is Not Responsible for Provisioning Types in ES
Most controversial one. User provisions ES on their own. Thanks to the API that doesn't require knowledge of types by CDL, we can do that with ES but not with PSQL.

> PROPOSAL:  
> In my opinion, we should update views to require users to specify types on all depth levels, 
> support it currently only in ES, and with time implement PSQL specific solution, using multiple tables.

### Validating Views on Addition
Having types map from schema to view, we can add validation to the process of creating a view.

## Type Storage
> DECISION:
> TLDR section refers to currently supported subset of json schema. In basics, we can store number, boolean, string and object (composed of 3 previously mentioned base types).
> Other types in json schema are supported by CDL but not materialization.

There are several options where types should be stored. In my opinion, it's view responsibility to store types of columns, and it's schema responsibility to store type of data.
We keep current schema format, with JSON compatible types (`string`, `number`, `boolean`, `one_of null`, `array`, `object`) and require users to specify type mapping for view.
In view, we'll support more granular types that are implicitly converted from JSON:
```
basic_type:
  f64 <- number
  i64 <- number # will panic when number is float
  string <- string
  bool <- boolean

complex_type:
  array[basic_type] <- array # will panic when array contains mixed types
  object(nested) <- object
  
nullability:
  additional field `nullable` that maps to `one_of: any | null` # `nullable = false` will have behaviour similar to .unwrap()
```

Once added to SR, view will emit a notification that's caught by OB, initiating materialization for it.
This notification will also be responsible for triggering database provisioning.

Such provisioning will follow mapping mentioned in [Type coverage](#Type-coverage).

Such system is open for extension and allows defining custom type casting, if that ever be a requirement.
In the future, backed by ConfS, we can introduce database-specific mapping, and even support both PSQL 9.6 and PSQL 13 independently.

## On-Demand Materializer

> DECISION:
> Materializer stays as is, map<string, string> in reality is map<string, json>, which, however imperfect, solves our issues.
> view - schema (and thus json) conversion is a surjection

On-demand materializer serves data over grpc, returning materialized rows mapping field name to stringified json at the moment.

We should update the mentioned format to return types that resemble view type definition in either a nested grpc message or map where keys are `.` delimited paths.
> SIDENOTE  
> '.' delimited paths are borrowed from ES, where this is how columns in internal indexes are stored.

### Nested Messages
We should map view types to grpc types via mapping:
```
i64 -> int64
f64 -> double
string -> string
bool -> bool
array[basic_type] -> `repeated`
object -> `message` # recursive

nullability: `optional`
```
This will require us to use `oneof` operator and will result in messages sent from one service to another being nested.

### Strings
We will send all data as string over grpc and fields will be in `path.to.deep.field` format (to ensure everything is flat), eg.:
```json
{
  "f1": {
    "f1_1": true,
    "f1_2": {
      "f1_2_1": "Hello"
    }
  }
}
```
Will result in:
```
{
    "object_id": "xyzz",
    "fields": {
        "f1.f1_1": { "value": "true", "type": "bool" },
        "f1.f1_2.f1_2_1": { "value": "Hello", "type": "string" }
    }
}
```
This however will increase network load of cdl.

> SIDENOTE:  
> We can skip "type" field and require users to use views to deduce field types.

### Opinion
We should aim for first option if possible, as it looks cleaner and provides better network performance.

# Other Options

## We Can Assume Everything is Nullable
As in title, CDL may assume that nullability is by default, and thus we have less work in case of PSQL materialization. We still have to be able to parse
`one_of: any | null` but now that's extraction of type information, rather than special behavior.

## Use Custom Schema Definitions With Custom Set of Data Types
Using custom implementation of JSON schema, or, what's mentioned in [rfc 0019][rfc-0019], creating custom standard may, in the future,
allow us to provide type mapping on the SCHEMA level, thus requiring less explicit view declarations (we'd only have to require mapping there, no implicit conversions from `number` to `f64`).
This custom schema would also allow us to provide more types as a base. 

## Instead of Adding Notifications to SR, Send Types With Each Request
Personally, I don't like this idea. It adds to the message payload a redundant piece of data, increasing the number of bytes that have to be sent through the network.

## Infer Types Instead of Specifying Them
This comes with a disadvantage. We can infer types on the first materialized row (however, everything must be nullable).
If in the field value `null` is present, we have to assign most generic type in materialization db (in PSQL it'd be JSON for example).

> SIDENOTE:  
> For PSQL we'd have to write inference mechanism on our own and any incompatibility would be caught at the moment of insertion.

# Decision

Document was changed to reflect decisions made during a meeting (23.08.2021) (mostly TLDR section).
We are providing support for basic numeric types, strings and booleans. We support nesting these values in objects.

We have moved arrays to later phase of development, their support requires additional RFC.
Same goes for nullability - everything is non-nullable, and changing this requires separate document.
From discussion arisen type casting problem, when users would like to store numeric values as strings, for better precision handling.
This also requires separate document and discussion.
All 3 of these "type" tackle what is defined "extension type" in this rfc.

We have to remember to emit errors on invalid casting from "json::number" to "view::i64/f64" and (if these values aren't checked earlier), to respective `ES` and `PSQL` types.

Within this rfc is mentioned topic of provisioning database on view insertion. This is moved to new, separate issue.

[json-schema-basic-types]: http://json-schema.org/draft/2020-12/json-schema-core.html#rfc.section.4.2.1
[rfc-0019]: ./0019_Simplify_Schema_Definitions_01.md
[wiki-adt]: https://en.wikipedia.org/wiki/Algebraic_data_type
[es-basic-data-types]: https://www.elastic.co/guide/en/elasticsearch/reference/current/mapping-types.html
[es-arrays]: https://www.elastic.co/guide/en/elasticsearch/reference/current/array.html
