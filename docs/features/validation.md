# Validation

Validation is mechanic of checking CDL input data against schema, in order to verify that it follows desired format.
CDL support validation in 3 places:
- schema-registry (active)
- data-router (pasive)
- validator (pasive)

Active validation is request-response. You can query `SR` in order to check whether given object follows it's schema.

Pasive validation is pipeline service, that passes messages only if they follow schemas.

# Schema-registry

There's ValidateValue route in `SR`. It can be used to check object agains it's schema. More can be found in `rpc` crate, in `proto` folder.

# Data-Router

Validation in `DR` is enabled via config:
```toml
[validation]
enabled = true
cache_capacity = 1024
```
Only messages that have proper schema are forwarded, rest is discarded. There are no notifications about discard at this moment.

# Validator

Separate service that can validate your data. It accepts single input and single output, validates messages agains schema in `SR` and discards invalid ones. There are no notifications about discard at this moment.
