#!/usr/bin/env bash

docker-compose down --remove-orphans
docker-compose up -d postgres kafka victoria_metrics
