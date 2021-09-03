# Front Matter

```
    Title           : Materialization computations
    Author(s)       : kononnable
    Created         : 2021-08-30
    Category        : Feature
    CDL Feature ID  : CDLF-00017-00
```

## Proposed Changes
CDL currently is very limited when it comes to processing data during the materialization process. In this RFC we propose adding some computable operators on numeric and booleans values. This will allow for simple computations to generate values returned by the materializer as well as used in filtering.

Types of materialization input values will be extracted from schema registry. That means that schemas on which views are based on needs to have a proper definition.
### Auto conversion:
All mentioned in this document operators take the same input types as output types. However, i64 values should be automatically converted to f64 type if the expected output is of f64 type.
### Errors
All of the mentioned operators should return errors on value types not matching schema, auto conversion errors and others, mentioned per operator type, as non-critical errors. That means that processing of the row should be aborted, an error should be reported, but processing of the whole process should be allowed to continue.

### Operators
Each operator defines types it can work on and lists of errors that can be returned. As operators' names should be self-explanatory operator descriptions are skipped. Errors regarding wrong parameter types are not mentioned here as they can always happen and are not related to specific operators or their usage.

#### Add
- Add(i64,i64) -> i64
Returned errors: Overflow
- Add(f64,f64) -> f64
Returned errors: None
#### Subtract
- Substract(i64,i64) -> i64
Returned errors: Overflow
- Substract(f64,f64) -> f64
Returned errors: None
#### Multiply
- Multiply(i64,i64) -> i64
Returned errors: Overflow
- Multiply(f64,f64) -> f64
Returned errors: None
#### Divide
- Divide(i64,i64) -> i64
Returned errors: Divide By Zero
- Divide(f64,f64) -> f64
Returned errors: None
#### Modulo
- Modulo(i64,i64) -> i64
Returned errors: Divide By Zero
#### And
- And(Boolean,Boolean) -> Boolean
Returned errors: None
#### Or
- Or(Boolean,Boolean) -> Boolean
Returned errors: None
#### Not
- Not(Boolean) -> Boolean
Returned errors: None
