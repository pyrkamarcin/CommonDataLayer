# Materializer - General
MG is a connector between CDL and the Materialization Database. It's used to convert CDL materialization format into per
database final result.

# Materialization databases
## PostgreSQL
TBD

## Elasticsearch
This version stores rows in an Elasticsearch database. Documents are stored in an index provided in
view's `materializerOptions.indexName` field.  
Their indices are hashes computed as follows:  
`for<every object_id that takes part in materialization of a given row> sort object_ids descending => compute SHA256 hash of that array`

[Configuration is available](../configuration/materializer-general.md).
