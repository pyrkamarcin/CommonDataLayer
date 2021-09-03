# CDL Performance Testing

## Glossary
* Playground - initial deployment, designed for use in CDL development.
* KPI - key performance indicator - a quantifiable measurement, often use in long-term statistics.

## Introduction
The purpose CDL internal performance testing process is to monitor CDL bottlenecks and allow us to see potential performance issues before they occur on CDL production deployments. Results don't match expected CDL performance in a production environment, as some assumptions done during the testing are not acceptable on production deployments. However, measuring expected CDL performance on a production environment can be done similarly, with some configuration changes(mostly proper production configuration for external services - Postgres, Kafka, etc.).

## Testing environment

#### CDL Carrier
Currently there is no production-ready configuration for CDL, kubernetes deployment is an example one, and baremetal deployment hosted via horust (systemd-like management tool) is targeted for development and general testing. For testing purposes, CDL will be deployed on either bare metal servers or virtual ones, using kubernetes management system.

Currently, for the playground, we are quite fine with using kubernetes deployment backed up by helm. We made sure that each service will own at least one cpu core during work, however some tests require at least 2 cores to tests parallel computing solutions, so this should be taken into consideration.

#### IO Sensitive Components
Components sensitive to the hardware configuration will have to receive its own machines, examples of such are the databases (Postgres, VictoriaMetrics, ElasticSearch). Those components will have unrestricted configuration, meaning it does not matter how it is set up on the destination node, however rules mentioned in the `Hardware Configuration` are still in effect.

In case of our initial deployment on playground, we decided to use Azure, as VM provider, and each of the databases received its own dedicated VM for exclusive usage. To such VM's we attached the dedicated SSD to have stable IO performance readouts.

#### Logging and Metrics
This section is split in two parts: message logging (as error messages or notification) and metrics (as statistics). Both of those systems are crucial for testing CDL properly.
* Metrics -  we are using Grafana, and as such each component have to expose its basic parameters for collection. Currently, the idea is that during performance testing we will have a separate component that will collect the metrics alongside Grafana and store it per test execution.
* Logging - for logging we are using elastic search server. As with the metrics, we are planning for the test-runner to be able to access the logs per test run and check for all issues associated with the run itself, log errors and warnings. Those will also be preserved.

### Hardware Configuration
Hardware configuration is designed to make the tests as repetitive as possible. As demanding as the testing is, CDL require multiple nodes to perform its computations in stable environment. What it means is:

* Network configuration is stable and is not changed across tests, that also means that there should not be changes in routing, routing hardware, network adapters, drivers nor topology across multiple tests.
* Node (as computing node) should not change its configuration. If there is a software update pending for the node, additional tests have to be run to determine if new software made an impact on tests. This may be especially important for the software that hosts the CDL, i.e.  container management system or dynamic libraries used directly or indirectly in CDL runtime.
* IO (as storage devices, hard disks or in worst-case scenario, NFS drives) configuration should be preserved. If hardware needs to be updated or replaced, tests on the same version and configuration of CDL but changed hardware should be done to determine how the new storage device compares to the previously used one.

** NOTE: for some components, we may test scalability in the future by adding another mirror node. The node will be on an identical machine (one of the free ones will be reused)

### Postgres

#### Machine
Standard DS3 v2 (4 vcpus, 14 GiB memory)
Additional SSD disk
** NOTE: we may test scalability by adding another mirror node. The node will be on an identical machine (one of the free ones will be reused)

#### Configuration
- Disabled WAL for CDL tables(`ALTER TABLE table_name SET UNLOGGED;`)

** NOTE: We're using the same Postgres database instance for Postgres Repository, Edge Registry, Schema Registry, and Postgres Materializer. **
** This is temporary measure, when testing process is ready we will decide if we need separate posgres instance for those components **

### Kafka

#### Machine
Standard DS3 v2 (4 vcpus, 14 GiB memory)

#### Configuration
Consumer:
- No performance related changes

Publisher:
- acks: all
- compression.type: none
- max.in.flight.requests.per.connection: 1

### RabbitMQ

#### Machine
Standard DS3 v2 (4 vcpus, 14 GiB memory)

#### Configuration
This component is planned but not deployed yet, it will be handled before perofrmance tests start.

### Victoria Metrics

#### Machine
Standard DS3 v2 (4 vcpus, 14 GiB memory)

#### Configuration
This component is planned but not deployed yet, it will be handled before perofrmance tests start.

## Results processing
CDL exposes information about runtime performance in form of metrics. They're effortless to use and provide much information to the user. Based on metrics(both CDL internals and Kafka/Postgres/k8s/machines related ones) user can determine what is the bottleneck. For automatic testing(monitoring bottleneck performance) we depend on more basic methods. However, if an automatic test reveals a possible problem, it's worth it for the developer to compare metrics on prometheus/grafana to see what's changed.

We should also check if there were any errors during the testing process. This can be done by checking the numbers of rows inserted/returned by the test after the test is finished. Each, even small mismatch in expected and actual numbers qualify for manual developer review.

We should store the following artifacts from testing:
- grafana metrics
- k8s cluster state (`kubectl get pods -o=wide`)
- CDL logs (only if there were any errors)
- dbs state (only if there were any errors)

## Test Focus
For now, we're testing the following processes. This list will grow over time.
- Data IO
  - Storage Layer Input
  - Storage Layer Output
- Materialization (WIP)
  - ondemand materialization
  - general materialization
- DB Shrinker
- historical data folding (WIP)

### General testing rules
- Before running the test, determine if kafka should be empty or pre-fed. Pre-feeding allows the test to help gather information about the component speed. Feeding it in place makes it more focused on real-life scenario.
- Test data should be generated once and reused, updated only once CDL input format changes. Generation should not be performed during the test.
- Tests should be run in the identical environment.
- Tests should be run 10 times, remove 3 results farthest from the rest, average rest to get a single value

### Data Input
Test options:
- average size of inserted messages (focus on the same size all the time)
- data size variation (small and big messages mixed in)
- message batching
- to work around IO write bottleneck, we can use /dev/null as an input connector.

Process description:
- Set up CDL
- Add CDL schemas definitions
- Start time measurement
- Load data from files to Kafka
  - optionally wait for the kafka to ingest everything, before starting the Data Router
- wait for CDL to emit either specific number of notifications or last message notification, whatever comes last
- Write down results - the time it took to insert data
- Check if actual number equals expected number of entries in sql table

### Data Output
Test options:
- average size of objects
- number of objects returned in a single request
- message fragmentation(number of fragments command service needs to merge before returning final objects)

Process description:
- Set up CDL, including Document Database feeding
- Fetch series of records from Query Router
- Write down results - responses time
- Check if the actual number equals the expected number of returned rows

### Materialization
Only on-demand materialization is tested, no database-specific materializers nor partial update engine is tested in those tests as of now(should be added in the future).

Test options:
- view complexness(simple, with relations, with filtering, with computations)
- average size of objects
- materializer type(on-demand, Postgres, elastic search)

Process description:
- Set up CDL
- Add CDL schemas, relations, views definitions
- Load data to SQL(object data, edges)
- Fetch data from on-demand materializer
- Write down results - response time
- Check if the actual number equals the expected number of result entries

### DB Shrinker
Test options:
- average size of objects
- different number of fragments per object

Process description:
- Set up CDL
- Add CDL schemas definitions
- Load data to SQL
- Start db shrinker job
- Wait for db shrinker job to finish
- Write down results - the amount of time db shrinker job was alive
- Check if the actual number is equal to the expected number of entries in the SQL table


# FAQ
- Why not measure latency?
CDL isn't a latency focused system, and measuring latency on a cloud system is a complex topic. Latency can change drastically based on service usage, and that change is not linear(more service traffic in some cases means smaller latency).
- Why no scaling testing?
CDL code-related bottlenecks are most likely to show on even simpler CDL deployments. Scaling is a complex topic, and it would make these tests much harder to implement.
