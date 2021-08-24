# Adding new tests

Please use following convention for UUID numbers
| Type        | UUID                                 |
|-------------|--------------------------------------|
| Object_id   | 00000000-0000-0000-0000-xxxxxxxxxxxx |
| Schema_id   | a0000000-0000-0000-0000-xxxxxxxxxxxx |
| Relation_id | b0000000-0000-0000-0000-xxxxxxxxxxxx |

Where xxxxxxxxxxxxx is incrementing value starting at 000000000001.

# Updating tests
To fix tests by accepting new version:

``` sh
patch --ignore-whitespace < *.new
```

or use interactive script:

dependencies: `colordiff`, `bash >= 4.0`

``` sh
./review.sh
```

TODO:
* [ ] Edge cases when View is invalid
* [x] Edge case for inner/left join 
