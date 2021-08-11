#!/usr/bin/env bash
killall api || echo "api"
killall data-router || echo "data-router"
killall query-router || echo "query-router"
killall schema-registry || echo "schema-registry"
killall command-service || echo "command-service"
killall query-service || echo "query-service"
