# Benchmarking for the Common Data Layer

Tools to benchmark response times for the CDL.


## Installation

All scripts are written using the [Crystal][crystal] language. Make sure you [install it][install crystal]
before proceeding. Once Crystal is installed, you will need to install dependencies with `shards install`.


## Running

The only script currently is `upload_to_rabbitmq.cr`, which uploads a large volume of random JSON data
to RabbitMQ (loaded from the [sample_json](./sample_json/) folder). More details about its usage can be found
by running `crystal scripts/upload_to_rabbitmq.cr -- --help`.
You can run the script with `crystal scripts/upload_to_rabbitmq.cr`.

The `upload_to_rabbitmq` script can also be run via Kubernetes with `kubectl apply -f pod.yml`.


[crystal]: https://crystal-lang.org/
[install crystal]: https://crystal-lang.org/install/
