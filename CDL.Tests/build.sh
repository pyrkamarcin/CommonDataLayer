#!/usr/bin/env bash

cp ../crates/rpc/proto/* .

for filename in ./proto-patches/*.patch; do
   patch < "${filename}"
done

dotnet restore || exit
dotnet build -o './publish' || exit

docker build -t cdl-tests:latest .
