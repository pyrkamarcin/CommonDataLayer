#!/usr/bin/env bash

cp ../crates/rpc/proto/* .

for filename in ./proto-patches/*.patch; do
   patch < "${filename}"
done

docker build -t cdl-tests:latest .
