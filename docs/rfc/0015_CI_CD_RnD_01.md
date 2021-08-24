# Front Matter

```
    Title           : CI/CD setup - R&D
    Author(s)       : Mateusz 'esavier' Matejuk
    Team            : CommonDataLayer
    Reviewer        : CommonDataLayer
    Created         : 2021-07-13
    Last updated    : 2021-08-06
    Category        : Research
    CDL Feature ID  : Not A Feature
```

#### Abstract

```
    The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL
    NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED",  "MAY", and
    "OPTIONAL" in this document are to be interpreted as described in
    RFC 2119.
```
[RFC 2119 source][rfc2119]

## Glossary
- PS - performance stack - the part of the ci that is responsible for hosting the performance part of the tests
- PT - performance test - tests related to measurements of release or build which was already tested against compliance with specification and developer tests
- DT - developer test - bulk of tests related to code/specification/project compliance (acceptance tests). Triggered  every commit to tests its viability, also ran on all types of releases. Currently, functionality is provided by GitHub actions.
- Reverse Proxy - both load balancing and security element in network communication, usually http - related. In our case, its nginx. Its main job is to provide TLS wrapping on the http layer (resulting in https TLSv1.3). While load balancing is not currently a problem. it is a security concern to allow users to authenticate over plain text, and thus component like this have to be introduced to help alleviate this problem.
- HW - hardware
- SW - software

## What Do We Want To accomplish By Introducing The CI/CD?
a.k.a.  What's the situation, and what are we trying to solve?

Currently, all our tests are being run on the GitHub public cloud. We do have, as an OSS project, a lot of flexibility in this matter, and GitHub provides us almost infinite resources to help with our development. However, being the generic system, we are using VM's and environments that we do not control fully. We cannot guarantee that tests that we want to do will run in the same environment and thus tests are not comparable with each other.

## Scope
This document only describes the changes and tooling related to the Performance-bound part of the CI. As described in the paragraph above, CI will be split it two dedicated parts. All the work related to R&D was around planning the CI with focus on performance part, taking only automated one into overall consideration. Reasoning behind it is that automated CI is already working to some degree and only needs quality of life improvements or minor expansions.

Measuring itself is currently free-floating problem that does not have to be immediately solved. To be specific, it's the problem of amount of work and path we will take given setup that we like, but it does not have to, at this point of time, to dictate setup itself.

### Solution
We decided that it would be the best idea to follow a hybrid solution for now. That means that we will keep GitHub actions for whatever we are doing right now, i.e. code quality and compliance testing, that can be run multiple times in parallel, and thus unlimited resources GitHub provides would be the best option for it. For performance testing, we will use the playground server stack which is configured, stable and persistent across tests. That also means that each test will be started in the environment as similar to each other as possible.

In addition, we will be able to perform server tuning and advise for best HW and SW configuration for CDL to run on.


### Architecture Considerations
Being a problem-specific solution, CI/CD setup needs to take following aspects have to be taken into consideration (the most important ones, list may be incomplete as its not the focus of this document):
- System IOps and bus performance
-- despite being run on virtualized environments, we can expect some soft limits to how many operations can be executed. This also includes network, disk and interbus communication (i.e. between controllers, logical units or memory, even thread communication to some degree). This means that while we can not compare the performance with another test done in different environments easily, it can be at least measured to some degree and reproduced. IO performance will have heavy impact on all services despite not every and each of them uses permanent storage like SSD/HDD.
-- The best solution for controlled environment would be a private bare-metal stack, however the budget for the project is not big enough to afford one, especially that the stack will have nothing to do during the working part of the week (right now we are considering running cron-like triggers on weekly basis, for example on weekends)
- Resources and resource localization
-- Resource localization can not be utilized in VM's since there is no NUMA controller and memory is not local, however we may be able to tinker with CPU binding in later stages which is kernel related functionality and is not present on containers. That also means that there is need for few non-containerized machines where we can run the service in debug mode and where we have some degree of control. We can not do this in github machines unfortunately since all of them are running as kernelless containers (non-nested KVM machines). The second topic to take into consideration when we are talking about localization is storage, especially on VM providers like azure or aws. We noticed before that there is a huge difference between running services on a specialized disk type, and if the disk is a virtual (usually network bound) resource or dedicated share in SSD storage.
- monitoring
-- monitoring services have to be erected on the same machine stack, since most of the services can provide either metrics or telemetry. It is crucial for performance testing to have monitoring services as, while only, its easiest method to measure whole system statistics without integration with the operating systems itself.
- locking mechanisms
-- we have to ensure that in case of performance testing, only one test will be running at the specific time. The locking mechanism can be anything as long as it functions properly and prevents simultaneous tests for being fired at the same time. This is also related to different test types running on the same PS.
- triggers and integrations with existing systems and procedures
-- Performance tests have to be triggered either manually or by a build. Integration with DT  environment is not necessary but welcomed. Measurements taken needs to be recorded and validated against other runs with the same HW/SW configuration, but that does not mean that the tests have to be triggered every release. As such having only manual triggers are entirely acceptable.
-- Measurements have to be collected and stored for comparison. Additional tooling may be required, since right now, not every part of existing metrics and/or observability systems are able to partake in automated testing (i.e. for example graphana can not be triggered to dump statistics from last n minutes)

## Tooling
in terms of availability and usability, the following tools were tested:
- Jenkins
- Buildbot
- Lamianr
- GoCD

#### Jenkins
It is a very versatile tool that allows for adjustments and modifications. Community is quite active and there is a lot of plugins for different systems and use cases. However, in our case, there were problems with proper configuration of the service itself. Numerous issues with HTTPS reverse proxy (among others) was noticed and prevented us from, for example, providing or changing configuration and communicating with the agents. Despite that, the setup can be adjusted to take that into account, for example by providing proper vpn connections, switching to plain http and locking the machine itself from the outside world. Another problem is that java errors are not easy to understand nor to fix.

##### Pros
- mature community
- everybody knows it to some degree
- low entry point
- configurable
- lot of plugins
- master - slave architecture
- configuration is separate from jobs

##### Cons
- a lot of configuration per job and around it
- needs reverse proxy
- config problems
- problems are hard to read/understand/fix, logs in in java format (stack dumps)
- issues with reverse proxy and https
- problems are not visible until encountered

#### Laminar
It is by far the simplest, yet most powerful tool yet. Its simplicity does not allow running agents as an entity, but it can be triggered and configured over network. There is notably less security issues that have to be looked after, since it is only serving read only notification panel in unencrypted http endpoint. Each Laminar instance acts as a worker, and each worker can trigger another worker. Jobs can be chained together, or depend on each other. Each job is either script or executable that is launched and maintained by Laminar to perform some sort of action, and as such, number of possibilities are countless and do not require too much effort to implement and maintain. Issues were raised however, over lacking functionality of the UI to display specific values, like throughput etc. It definitively can be done, however exact time to provide UI modification and functionality is unknown.

##### Pros
- dead simple
- everything is a form of script (binaries are also allowed)
- very easy configuration
- little to no security considerations
- very easy to maintain
- work node architecture (master is worker, nodes can trigger each  other)
- responsive gui
- does not need reverse proxy to work while maintaining security
- pre/post actions allowed, also as scripts
- configuration is separate from jobs

##### Neutral
- GUI is read only
- webhooks need additional tooling (like rust's 'snare')
- if advanced kind of report display is needed, separate

##### Cons
- very simple and limited GUI
- tests can either pass or not (binary)
- more complex problems like chaining or waiting for other jobs requires some sort of hacks which will vary in degree of complexity depending on encountered problems
- probably will require vpn, vswitch, bridge or tunnel to communicate with other nodes

#### BuildBot
This is a simple yet responsive builder framework written in python. All configurations and build descriptions are also written in python. It allows for master-slave (i.e. build agents), and multi-master configurations, however all of the jobs are defined in the master's configuration file. This poses problems for us, since it means that we will have to adjust most aspects of the builds ourselves, a.k.a. code it in. Since UI is also very simple, the same issues related to it from Laminar stands for BuildBot too. While not overly complicated, to bring it to proper usability, a significant amount of work will be needed.

##### Pros
- master-slave architecture
- rather simple
- responsive gui
- test written in python

##### Neutral
- its only a framework
- multi-master configuration, although not trivial to introduce

##### Cons
- needs reverse proxy
- tests are located in master's config file, while those can be separated and its easy to maintain it, it will be hard to test those out, since there is nothing testing the test configuration before it is deployed to the ci (jenkins for example can have not-working tests, while BuildBot will simply refuse to load on error)

#### GoCD
While giving good impression at first, being fast and responsible, there are multiple issues with configuration and running. There is problem with injecting proper environment variables. More problems were present while interacting with 3rd party plugins, like elastic agents. Their configuration was not trivial and errors were not conveyed properly, so we were unable to fix the problems. Given that, it permits secure transport to the native agents running in different cluster with relative ease.

##### Pros
- nice looking and responsive UI
- master-slave configuration
- easy to setup security both master-slave and master-user (gui)
- configuration is separate from jobs

##### Neutral
- gui and configuration is not exactly straightforward, while not bad, its also not trivial
- despite the name, its not in Go...

##### Cons
- ... its in java
- problems are hard to read/understand/fix, logs in java format (stack dumps)
- needs reverse proxy
- everything works until it doesn't, same problem as with jenkins. If there is a bug its only visible when you want to use the functionality. To check configuration, you need to literally click over all menu and situations to check if everything works and does not result in stack dump, although problems encountered were not at least related to reverse proxy config.

## Finishing Touches
Currently, it is not required to take any action regarding CI. Performance testing is not scheduled to be done soon, and this priority is low. There is still time to check other tooling options, and in the case it happens, this document will be expanded with results.

I decided that there is no need to explain current DT stack, since it's not the scope of this document, and besides that, it works for now. In case any changes are required to DT, a separate document will be provided linking, this one, hopefully, as a reference material.

## References
[rfc2119]:https://www.ietf.org/rfc/rfc2119.txt