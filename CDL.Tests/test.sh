#!/usr/bin/env bash
set -ex

## run from repo main dir

kubectl delete pod cdl-tests || echo ""

cd ./CDL.Tests
source ./build.sh
cd ..

kubectl apply -f ./CDL.Tests/pod.yml
sleep 5
kubectl logs cdl-tests --follow 
sleep 5
[[ -z `kubectl get pods -o=jsonpath="{.items[?(.status.containerStatuses[0].state.terminated.exitCode!=0)]['metadata.name']}"` ]]

