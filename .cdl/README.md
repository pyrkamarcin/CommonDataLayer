# Configuration

This directory contains exemplary configuration files for cdl.
* development-kafka/ for kafka + postgres setup
* development-amqp/ for rabbitmq + postgres setup
* development-grpc/ for grpc + postgres setup

There's no default setup, user when setting up the CDL should generate one via `cargo xtask config-generator` or 
use either of aforementioned configurations.
In order to select given configuration, CDL_CONFIG environment variable has to be set to specific directory, eg.:
`CDL_CONFIG=.cdl/development-kafka ./api`
