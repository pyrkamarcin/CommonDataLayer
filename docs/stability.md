# Stability of Common Data Layer

## Glossary
Stable API - breaking change requires MAJOR version bump,
Experimental API - breaking change does not requires MAJOR version bump.

## Motivation
Common Data Layer is growing rapidly. It means that some parts of its API may change over time.
To prevent bumping MAJOR version every release (or slowing down development by stabilizing every feature) team decided to pick API parts that are mature and should not change very often and stabilize it. The rest is considered unstable/experimental - you can play with it but it might be prone to errors or major changes.


## Legend
⛔ - Experimental API
⚠ - Partially stable API
✅ - Stable API

## Stability

### Query Router ⚠
- Retrieve Single Object ✅
- Retrieve Multiple Objects ✅
- Retrieve Schema Objects ✅
- Execute Raw Query ⛔

### Data Router ✅
Data Router uses unified generic consumer and publisher wiht CDL_IM as a message format.

### Schema Registry ⚠
- Add Schema ✅
- Update Schema ✅
- Get Schema ✅
- Get All Schemas ✅
- Watch All Schema Updates ✅
- Add View To Schema ⛔
- Get Full Schema ⛔
- Get View ⛔
- Get All Full Schemas ⛔
- Get All Views Of Schema ⛔
- Get All Views By Relation ⛔
- Get Base Schema Of View ⛔
- Validate value ⛔

### Query Service Document Storage ⚠
- Query Multiple ✅
- Query By Schema ✅
- Query Raw ⛔

### Command Service Postgres ✅
Command Service uses unified generic consumer and publisher wiht CDL_IM as a message format.

### GraphQL and Management Panel ⛔
Whole graphQL API is unstable

### Edge Registry ⛔
- Add Relation ⛔
- Get Relation ⛔
- Get Schema By Relation ⛔
- Get Schema Relations ⛔
- List Relations ⛔
- Validate Relation ⛔
- Add Edges ⛔
- Get Edge ⛔
- Get Edges ⛔
- Resolve Tree ⛔

### Materializer Postgres ⛔
Whole materialization is concidered experimental

- Validate Options ⛔
- Upsert View ⛔

### Materializer Elasticsearch ⛔
Whole materialization is concidered experimental

- Validate Options ⛔
- Upsert View ⛔

### Materializer On-Demand ⛔
Whole materialization is concidered experimental

- Materialize ⛔

### Object Builder ⛔
Whole materialization is concidered experimental

- Materialize ⛔

### Query Service Time Series ⛔
- Query By Schema ⛔
- Query By Range ⛔
- Query Raw ⛔

### Command Service Time Series ⛔
Time Series database is experimental

### Command Service Elasticsearch ⛔
Elasticsearch support is experimental

### Partial Update Engine ⛔
Whole materialization is concidered experimental

