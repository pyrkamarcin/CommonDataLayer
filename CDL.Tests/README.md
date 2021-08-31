# Common Data Layer Testing Framework

Testing framework for [Common Data Layer][cdl] , implemented in the [.Net5][net5] language.

## Prerequisites

CDL and infrastructure have to be deployed somewhere. Please see our docs - [deployment][deployment]. 

Below are listed environment variables that are required to successfully launch the tests.


| Env variable name | Value |
|---|---|
| CDL_EDGE_REGISTRY_ADDRESS | http://localhost:50110 |
| CDL_MATERIALIZER_GENERAL_POSTGRES_ADDRESS | http://localhost:50203 |
| CDL_MATERIALIZER_GENERAL_ELASTICSEARCH_ADDRESS | http://localhost:50213 |
| CDL_MATERIALIZER_ONDEMAND_ADDRESS | http://localhost:50108 |
| CDL_QUERY_ROUTER_ADDRESS | http://localhost:50103 |
| CDL_QUERY_SERVICE_ADDRESS | http://localhost:50201 |
| CDL_SCHEMA_REGISTRY_ADDRESS | http://localhost:50101 |
| CDL_KAFKA_BROKER | localhost:9092 |
| CDL_KAFKA_DATA_INPUT_TOPIC | cdl.data.input |
| CDL_KAFKA_EDGE_INPUT_TOPIC | cdl.edge.input |
| CDL_SCHEMA_REGISTRY_DESTINATION | cdl.document.1.data |
| CDL_ELASTICSEARCH_NODE | http://localhost:9200 |

Before you start, you have to have the latest proto files. Tests solution requires proto files placed in the solution folder. 


To do it you can run build.sh script to copy and patch proto files, and also docker image with tests will be created.

## Launch tests
If you use sources 'dotnet test'

If you use build artifacts. 'dotnet vstest CDL.Tests.dll'

Run docker container with tests image. 

'docker run --network host --env-file cdl.env cdl-tests:latest'

[net5]: https://docs.microsoft.com/en-us/aspnet/core/?view=aspnetcore-5.0
[cdl]: https://epiphany-platform.github.io/CommonDataLayer/
[deployment]: https://epiphany-platform.github.io/CommonDataLayer/deployment/index.html
