
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
