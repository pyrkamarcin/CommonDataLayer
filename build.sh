#!/bin/bash
set -ex

# ENVS:
# Set only if you want to push images to remote repository
# CDL_REPOSITORY_PATH=epiphanyservices.azurecr.io/
# CDL_VERSION=0.1.0
# CDL_PUSH=true

array=( data_router command_service query_router query_service leader_elector schema_registry document_storage upload_to_kafka upload_to_rabbitmq )
DOCKER_BUILDKIT=1
for i in "${array[@]}"
do
	docker build -t ${CDL_REPOSITORY_PATH}cdl_${i}:${CDL_VERSION:-latest} --build-arg BIN=${i} --build-arg ENV=${ENV:-PROD} .
done

if [[ ! -z "$CDL_PUSH" ]] 
then
	for i in "${array[@]}"
	do
		docker push ${CDL_REPOSITORY_PATH}cdl_${i}:${CDL_VERSION:-latest}
	done
fi
