# Front Matter

```
Title           : Materialization field types support
Category        : Feature
Author(s)       : Łukasz Biel
Team            : CommonDataLayer
Created         : 2021-12-08
Deadline        : 2021-12-08
CDL feature ID  : CDLF-00018-00
```

Related:

* https://github.com/epiphany-platform/CommonDataLayer/issues/656

# Glossary

* TS - timeseries
* QSTS - `query-service` for timeseries
* QS - `query-service` for documents
* ES - elasticsearch
* PSQL - PostgreSQL
* SR - Schema Registry
* Blob - `any` data, eg. pdf report, ms office spreedsheet, jpeg image, that CDL shouldn't preprocess in any way
* ADT - algebraic data type ([wiki][wiki-adt])
* ConfS - configuration service, central repository of CDL configuration, currently in rfc phase.

# Summary

TODO

# Configuration layer

At the moment of writing this rfc, data is stored in cdl in the following format:

#### Documents

JSON objects. They are not indexed by any field and querying for only parts (fields or sub-documents)
requires fetching whole JSON and then performing operations on it.

#### Timeseries

Timeseries data is managed by databases, often requires special queries on QSTS side, but most likely can query for
specific fields.

#### Blob

Blobs are stored as is. You cannot perform operations on it.

> SIDENOTE:  
> Other storage types are included for future reference. Currently, materialization is limited to Document Store.  
> If that was to change, new document would have to be created, exploring the integration.  
> From this point on we are gonna focus on document storage only.

JSON has fairly limited type system. We can read more about it in [JSON Schema spec][json-schema-basic-types]. JSON
schema is relevant there due to it being used in SR.
> SIDENOTE:  
> While it's used in SR, it's not really "used". It is planned to be incorporated in validation.
> [rfc 0019][rfc-0019] talks about simpler format, as a replacement and for the sake of Materialization I will explore expanding it later.

| type name | description |
|---|---|
| number | any number of any precision |
| string | Unicode string of characters of any length |
| boolean | "true" or "false" keyword |
| null | a null value, or, to be precise, lack of thereof |
| object | a json key-value, unordered, map |
| array | an ordered collection of records, can hold mixed types |

JSON Schema allows also to construct "one_of" ADTs. This is slightly controversial, as neither PSQL nor ES support that kind of construct for most of their types.
With one exception. Nullability is often described in json schema with `"one_of": any | null` and both databases house nullability feature.

# Materialization storage layer

## Postgresql

Before I start listing postgres data types, there's one big issue to mention - at this time we don't have specified
version of PSQL that we support. There are minor discrepancies in both behavior and (more importantly to us) type
system, between 9.6 and 13.

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

In the diff excerpt we see, that 13 introduces new types `macaddr8` and `pg_snapshot`, and deprecates `txid_snapshot`. I
have to mention also, that `pg_snapshot` is not present in PSQL 12, but `macaddr8` is.

That out of the way, we don't really know if we should support all postgres types to begin with.

| name | aliases | description |
|---|---|---|
|bigint|int8|signed eight-byte integer|
|bigserial|serial8|autoincrementing eight-byte integer|
|bit [ (n) ]| |fixed-length bit string|
|bit varying [ (n) ]|varbit [ (n) ]|variable-length bit string|
|boolean|bool|logical Boolean (true/false)|
|box| |rectangular box on a plane|
|bytea| |binary data (“byte array”)|
|character [ (n) ]|char [ (n) ]|fixed-length character string|
|character varying [ (n) ]|varchar [ (n) ]|variable-length character string|
|cidr| |IPv4 or IPv6 network address|
|circle| |circle on a plane|
|date| |calendar date (year, month, day)|
|double precision|float8|double precision floating-point number (8 bytes)|
|inet| |IPv4 or IPv6 host address|
|integer|int, int4|signed four-byte integer|
|interval [ fields ] [ (p) ]| |time span|
|json| |textual JSON data|
|jsonb| |binary JSON data, decomposed|
|line| |infinite line on a plane|
|lseg| |line segment on a plane|
|macaddr| |MAC (Media Access Control) address|
|macaddr8| |MAC (Media Access Control) address (EUI-64 format)|
|money| |currency amount|
|numeric [ (p, s) ]|decimal [ (p, s) ]|exact numeric of selectable precision|
|path| |geometric path on a plane|
|pg_lsn| |PostgreSQL Log Sequence Number|
|point| |geometric point on a plane|
|polygon| |closed geometric path on a plane|
|real|float4|single precision floating-point number (4 bytes)|
|smallint|int2|signed two-byte integer|
|smallserial|serial2|autoincrementing two-byte integer|
|serial|serial4|autoincrementing four-byte integer|
|text| |variable-length character string|
|time [ (p) ] [ without time zone ]| |time of day (no time zone)|
|time [ (p) ] with time zone|timetz|time of day, including time zone|
|timestamp [ (p) ] [ without time zone ]| |date and time (no time zone)|
|timestamp [ (p) ] with time zone|timestamptz|date and time, including time zone|
|tsquery| |text search query|
|tsvector| |text search document|
|txid_snapshot| |user-level transaction ID snapshot|
|uuid| |universally unique identifier|
|xml| |XML data|

Out of this list, we could use mapping `json::string` -> `psql::text` and `json::boolean` -> `psql::boolean`, and that's it.
Everything else has to be handled per case. Inference can't really work, as `json::number` may map to `real` or `int` or `bigint`.
Nullability is achieved via additional flag on column: `NOT NULL`.
Right now as a workaround we use JSON on all columns, but this is a temporary solution. And end user will require better management of database.

## Elasticsearch

| Elasticsearch type |
|---|
| null |
| boolean |
| byte |
| short |
| integer |
| long |
| double |
| float |
| half_float |
| scaled_float |
| keyword |
| text |
| binary |
| date |
| ip |
| object |
| nested |
| types |

This is a list of SQL mapped types, it's not fully exhaustive, more can be found on [ES documentation page][es-basic-data-types].
As we can see, again there's greater granularity in ES than it is in JSON.

Additionally, ES is able to deduce types based on insertions (first insertion to a given index generates type mapping).

# Proposal

## Type coverage
We should aim to initially, as an MVP, support minimal subset of common types.
Also, as an extension, we should add new types as needed/when necessary in the future.
For database specific types, eg. `psql::timestamp`, when there's ConfS in place, we should be able to provision CDL with concrete list of supported types.
Also - it can be user configurable.

I propose to use `String`, `bool`, `i64` and `f64` as base types for first iteration, with addition of nullability, arrays and objects composed of the base.

PSQL mapping in this case:
```
`String` -> `psql::text`
`bool` -> `psql::bool`
`i64` -> `psql::int8`
`f64` -> `psql::float8`
`null` -> COLUMN IS NULLABLE
`array` -> type[]
`object` -> JSON
```

ES mapping:
```
`String` -> `es::text`
`bool` -> `es::boolean`
`i64` -> `es::long`
`f64` -> `es::double`
`null` -> See note below
`array` -> See note below
`object` -> supported by default, raises some ES specific problems
```
> SIDENOTE:  
> Arrays and nullability are treated the same in ES, a field can contain 0 or more entries. [reference][es-arrays]

Also, I propose to disallow mixed type arrays on schema level. If in future a client rises need for those, we may consider `array[JSON]`.

As for `object` type, there are few options on how to approach this:
#### ES requires "deep" types. PSQL stays with JSON.
It means that we have differing implementations between two databases, where one requires "type provider" to declare what is stored within object and the other stores `JSON` as is. You

#### ES requires "deep" types. PSQL uses Record type.
We change PSQL to use a type that requires user to specify types in deeply nested objects, explicitly in view definition. This will mean ES and PSQL use same strategy.

#### We disallow for deep objects in both.
Everything has to be mapped to flat. This is probably impossible for clients to accept.

#### We store deep objects as strings in ES.
This would mean an object is just a blob. We lose ES indexing ability, but we gain similar behavior to PSQL.

#### We allow ES to deduce type on it's own.
This may be a bit error prone and require extra testing, but in theory we could define everything up to depth = 1, and leave rest to the database.

#### CDL is not responsible for provisioning types in ES.
Most controversial one. User provisions ES on their own. Thanks to the API that doesn't require knowledge of types by CDL, we can do that with ES but not with PSQL.

> PROPOSAL:  
> In my opinion, we should update views to allow users to generate "deep" objects, support it currently only in ES, and with time implement PSQL specific solution.

## Type storage
There are several options where types should be stored. In my opinion it's view responsibility to store types of columns and it's schema responsibility to store type of data.
This means, that we'll keep current schema format, with JSON compatible types (`string`, `number`, `boolean`, `one_of null`, `array`, `object`) and require users to specify type mapping for view.
In view, we'll support more granular types, that support implicit mapping between them and the originals:
```
basic_type:
  f64 <- number
  i64 <- number
  string <- string
  bool <- boolean

complex_type:
  array[basic_type] <- array
  object(nested) <- object
  
nullability:
  additional field `nullable` in view field definition when schema specifies `one_of: null`
```

Once added, view will emit a notification that's caught by OB, initiating materialization for it.
This notification will also be responsible for triggering database provisioning.

Such provisioning will follow mapping mentioned in [Type coverage](#Type-coverage).

This system is open for extension and allows defining custom type casting, if that ever be a requirement.
In the future, backed by ConfS, we can introduce database specific mapping, and even support both PSQL 9.6 and PSQL 13 independently.

# Other options

## We can assume everything is nullable
As in title, CDL may assume that nullability is by default, thus we have less work in case of PSQL materialization (no `NOT NULL` on fields), 
and we don't have to handle special case of `one_of: null` by simply disallowing the construct in JSON schema.

## Use custom schema definitions with custom set of data types
Using custom implementation of JSON schema, or, what's mentioned in [rfc 0019][rfc-0019], creating custom standard may, in the future,
allow us to provide type mapping on the SCHEMA level, thus requiring less explicit view declarations (we'd only have to require casting there).
This custom schema would allow us to provide more types as a base. 

> SIDENOTE:  
> Still, we'd have to find a common subset of types between all our databases, or perform casting on QS layer.

# Notes

### Notes Summary

This rfc aims to provide groundwork for materialization types support, it is mapping output tables to different types
than only JSON. # todo

### Notes JSON

Where to store type information? . What about mapping then? Should types be sent with each
materialization message? Should materialization request contain column types? Who does mapping/converting then? Can type
info be stored in schema? Can type info be stored in view? Do we expect type conversions in views? What about `one_of`
json schema construct? What about nullability? What about `object` and `array`. What about rust - db bridge? Does user
need to know what is materialization target?
---

### Notes PSQL

https://www.postgresql.org/docs/9.6/datatype.html
https://www.postgresql.org/docs/13/datatype.html

Are we interested in all types? Text vs varchar vs char[].

Important - which version of postgres we support? Do we support multiple versions of Postgre? Are there differences
between supported types? Do we support types that are added by plugins (eg. UUID).

---

### Notes ES

https://www.elastic.co/guide/en/elasticsearch/reference/current/sql-data-types.html
https://www.elastic.co/guide/en/elasticsearch/reference/current/documents-indices.html
Does ES even support types (yes)? Should we provide mapping? When? ES can infer types.


[json-schema-basic-types]: http://json-schema.org/draft/2020-12/json-schema-core.html#rfc.section.4.2.1
[rfc-0019]: ./0019_Simplify_Schema_Definitions_01.md
[wiki-adt]: https://en.wikipedia.org/wiki/Algebraic_data_type
[es-basic-data-types]: https://www.elastic.co/guide/en/elasticsearch/reference/current/mapping-types.html
[es-arrays]: https://www.elastic.co/guide/en/elasticsearch/reference/current/array.html
