#  Front Matter

```
    Title           : Configuration Serive
    Author(s)       : Mateusz 'esavier' Matejuk
    Team            : CommonDataLayer
    Reviewer        : CommonDataLayer
    Created         : 2021-07-27
    Last updated    : 2021-08-19
    Category        : Feature
    CDL feature ID  : CDLF-0000F-00
```

#### Abstract

```
     The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL
     NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED",  "MAY", and
     "OPTIONAL" in this document are to be interpreted as described in
     RFC 2119.
```
[RFC 2119 source][rfc2119]


### Glossary
CFGS          - Configuration service
CCFGS         - Client of configuration service
Deployment    - in context of this document, its and abstract group assembled from CommonDataLayer services working together in abstract but defined environment.

### Features
* CDLF-0000B-00 - CDL Feature - Access Groups.

### Front matter
In the following document, there are places where arbitrary evaluation of the solution is in scale of 10 points. While trying to be as objective as possible, I can't deny that this evaluation may be biased by my experience with similar problems.  Please use them as a reference only.

During this document creation, only one feature was being considered directly (mentioned in `Features` section). Besides that, only future-proofing against possible features and further expansion was taken into consideration.

### Introduction
It is required for CDL to handle dynamic configuration and provide configuration point for the deployment.

Independent of the implementation chosen, below are the major restrictions the configuration service has to comply with.

#### - Requirements
- providing central configuration repository
- providing configuration for specific services
- allowing services to be configured by group or instance
- providing at least a minimum degree of the security (protocol security, i.e. no config changes from unauthorized sources, message verification etc.)
- configuration provisioning and configuration service should be abstract from the way CDL is deployed and should not limit deployment in any way.
- providing dynamic configurations
- providing proper authentication and authorization of configurations

#### - Optional Requirements:
- service discovery and autoprovisioning
- High level administration of services

#### - Additional Assumptions
- solution has to provide api that is permissive to 3rd party services which may be written in different languages.
- user-facing security is a separate issue which is not addressed here. However, the solution has to allow for it.

#### Additional notes
As this is a general RFC, most functionality that is taken into consideration does not have to implemented right away. However, it is crucial to both describe it and take it into consideration during feature planning.

#### - Dynamic configuration
This feature requires us to notify interested services that there is some change or event to act upon. This does not distribute the configuration itself, only notifies the services that such change happened. If the service has proper privileges and is interested in such event, it will have to contact the configuration service itself to get this information. Examples of such events are (paraphrasing of course :)) :
* application restarts:
  - CS -> CFGS: "hey, stuff happened, i restarted, i have new port and address"
  - cfgs -> broadcast : "there is an update of routing information involving CS"
  - DR1, DR2 ... -> CFGS : "gimme new routing info!"
* configuration changes (example with service permissions):
  - user -> CFGS : "okay, we are changing from Postgres to elastic search"
  - CFGS -> broadcast : "there is a change in configuration for materializers"
  - M-G1 -> CFGS : "what changed?"
  - DR1 -> CFGS : "what changed? (technically shouldn't happen)"
  - CFGS -> DR1 : "nothing for you (you do not have permissions to see that change)"
  - CFGS -> M-G1 : "restart with new plugin, now you are working with elastic search"
* initial discovery (CFGS is already up) and tag grouping:
  - DR1 -> CFGS "hello, i am DR" (connection to CFGS estabilished)
  - CFGS -> DR1 "here is your config, you are configured as `[{ 'DR', 'tag-cold-storage'}, {'DR','tag-hot-storage'}]`"
  - CFGS -> broadcast "new change involving tags: `[{ 'DR', 'tag-cold-storage'}, {'DR','tag-hot-storage'}]`"
  - webapi -> CFGS : "iam interested in tag sets: `[DR+tag-cold-storage]`, gimme update"

While notifications are optional, its availability can influence deployment's behavior. For example, in case of lack of dynamic configuration, the whole deployment will have to be reset and started anew, presumably by administrator or management api (i.e. systemd or kubernetes). The same have to happen in case of nonexisting notifications in some cases of dns-less deployments, for example when service will crash and restart with different port, or in case of hot-adding services during the runtime.

High level service management is something is not utilized yet, but may be necessary to provide it in the future. The Simplest example of which is to restart the service on demand by operator (administrator, for example). Such behavior is not required right now, however the choice of solution should be influenced by its possibility to support that feature in the future.

#### - Security
##### Important!
In context of this document, we will only discuss protocol security inside cdl network. This means that the protocol inside the CDL stack have to be properly secured, as in the future, any incursion in configuration exchange may have dire consequences.
* Authentication - This means that we need to communicate with secure connection and the message authenticity has to be confirmed.
* Authorization - This means that we need to act only on the messages from a known source that has the ability to provide us with such message (i.e. central server, not another node)

##### User Interface
User-facing security features are omitted, but solution have to provide space for its implementation. We need to make sure that services will react only on reconfiguration requests from a trusted source, and prevent man-in-the-middle injections.

##### Securing the messages
Various security options have to be explored for future use. Authentication and authorization are both related to the general security (and as such to the nature of configuration service), and to separate feature - "Access Groups". It requires providing "trust but verify" mechanics for both client and server. That assumes both the sender and the payload can be verified against known providers. What it entails is:
- Configuration Service has to have:
  - CFGS's general private key
  - operator's private key
  - list of secrets bound to particular functionalities or access groups
- Client of configuration service has to have:
  - operator's private key
  - CFGS's public key
  - client's private key
  - secret used to identify the permissions of the client,

Examples of the technologies that support this requirements are:
* TLS (although using self-signed certs internally is an abomination imo)
* PGP

Examples of libraries that can help with implementation are:
* libnoise
* libp2p
* libsodium

Example of functional composites that can fullfill those requirements are:
* (AES-GMAC + Diffie Hellman + ec25519)

Additionally, the payload have to be protected by hash authentication code (i.e. AES-GMAC) in the future. This is the mechanism that protects both the message and its hash with control sum, which is first appended then encrypted together with the message.

Access Groups related functionality have to take into consideration that the same type of the service can form a "usage group". It means that there may be for example a deployment with two different Data Routers groups with different permission, features or functionalities and changes related to those services have to be addressed separately.

### Formats
With current design, there is no specific formats to be considered. One note is that configurations should be served either as MessagePack or JSON if possible.
- With usage of the libp2p all request-response messages will be carried using lenght-value format, where value is a payload, and length is its payload's length. In case of publisher-subscriber model, messages are coded/decoded automatically without user's interference. However, payload is the same as in the case of the request-response behavior.
- With HTTP model, messages are encoded using HTTP protocol definitions, while payload's format is left to the implementor to define. Headers should not be used.
- With any other library, the way of encoding messages are also left to the implementor to figure out.

An important limitation is that multiple languages have to be able to use the communication method being proposed, as configuration have to be served potentially to a number of services, which may or may not be written in rust, including 3rd party integrations.

Additionally, we concluded that there are few optional fields that are required for service implementation and grouping. Types such as list of tags that are affected by update, and a list with tags and its relations to each other are required to filter which services have to be updated. However, minimum implementation can skip filters and opt out only for a tag list. An example of a tag list is:

Client1:
```
{ "tags": ["postgres_public", "postgres_non_public"]}
```
Client2:
```
{ "tags": ["victoria", "timeseries", "postgres"]}
```
Server:
```
{ "tags": ["postgres_cold", "postgres_non_public"]
```

Note: if Server sends notification with those tags, only Client1 should react, at least until filters are not implemented. With simple tags, only matching tags are going to react. However, tag configuration needs to be cautious not to mix allowed tags. Further implementation can change the format to allow for secrets, so only authorized services can use specific tags.

A second option that is viable in that case is topic separation, although the application will have to listen to multiple topics, which may not be optimal depending on the implementation and library behavior. However, little research was done to prove that, so it is left for implementor to decide.

Configuration Service will be treated as an internal API and changes to it will not be considered breaking change.

It is acceptable to define API during early development stage and change or expand it later.

Integration with GraphQL or management panel has to be considered in later stages of the implementation to provide API for human end users.

## Solutions
All solution below will be described in detail along with its gains and losses.

### Static Server
Configuration is accessible as a URI tree served by static server. Configuration is immutable, and lives as long as the deployment (i.e. in case deployment have to change, everything is scrapped and set anew with new config). In that case, configuration can be served by any HTTP or TCP server, written in any language, including but not limited to, bash, zig, rust or an already existing tool (nginx, etcd) can be used. Any means and protocols available to serve the content are permitted for use and depends on implementor approach.

Chosen approach for static server is left out of this document, as different tools and approaches have to be tested and taken into consideration. In case this solution is chosen, additional RFC have to be written, containing things like evaluation and benchmarks.

There are few options related to the way deployment is handled, for example - etcd, that is available for Kubernetes deployments almost out of the box. Unfortunately, this option is not promising since it would create alternative paths between deployment types. To explain this properly: the service has to be deployment-neutral and won't work, for example, in case of bare-metal deployment or in resource-limited applications and in those cases alternative tool and/or solution have to be used.

positives:
- easy to implement/setup
- simplicity, it's really hard to mess it up
- quite a few ways to achieve this solution
- no replication problems, since it's fully centralized (one instance)

neutral:
- no notifications (there are no changes either way, so those are not necessary, but still, it's a limitation)
- no security
- no authentication

negatives:
- inflexible
- immutable
- cannot support reconfiguration as there is no notification medium (in most cases)
- it will be increasingly harder to implement further features upon this solution
- heavy load on the server in case of large deployments

Summary:
- relative work: 1/10
- maintenance work (periodic): 2/10
- flexibility: 1/10
- features: 1/10
- work needed to expand in the future: 10/10

### VCS-based Configuration
This is quite controversial topic to explore, however tools described here are used for similar purposes. Please consider it as more of a variant of previous solution than a standalone one. This solution was last-minute addition and was not tested in the field.

VCS stands for version control system, and the best and main example of which being explored here is git. A configuration repository can be configured and provided for the deployment, which will be then served as a static content, similarly as described in the former solution. That means that joining those two methods together retains some features of static server, providing some dynamic configuration options, whilst not requiring too much additional work. The biggest problem here is the trade-off between maintainability and implementation work, which is considerably bigger than in other solutions.

One significant problem to resolve is how to handle notifications. At first glance, since git provides an embedded mechanism that can provide HTTP request/notifications, the overall solution seems to be working out of the box, however after more profound analysis, there are few problems that may pose issues. Below are the biggest two i was able to explore:
- One of the requirements is being deployment-neutral, which entails that not every deployment will be able to use DNS resolution to find the services. That means webhook will have to parse configuration to know where the service is and how to notify it.  That also requires configuration to be in a format that a specific hook can understand and parse out the information it needs, which also means that for each config (format, not value) update there will be a hook update that will add code to understand the new format.
- There is hardly any security in http request. This can be alleviated by providing scripts that will wrap the request in some kind of format that will do it, but there is no way to enforce that rule.

While hook is a executable script or binary which can be changed at will, expanding functionality in the future will not be trivial. This solution, while trying to make up for lacking functionality of static server, incredible amount of work if any additional behavior is requested.

Positives:
- it provides everything that static server provides
- allows for limited degree of mutability
- provides limited options for access management
- no replication problems, since its fully centralized (one instance)

negatives:
- limited security (problems with security administration)
- no authentication / non-trivial implementation
- no easy and elegant way of providing notification support
- it will be increasingly harder to implement further features upon this solution

Summary:
- relative work: 3/10
- maintenance work (periodic): 5/10
- flexibility of the solution: 3/10
- available features: 3/10
- work needed to expand in the future: 6/10

### Central Server
It is a mutation of the previous method. Server is a custom service or tool that serves configuration in request-response model. Additionally, the server can support both mutability and notifications. Services interested in the configuration will have to ask for server directly for required configuration. Whatever the protocol would be (rest/grpc/http/tcp/udp) is up to the implementor, since high-level behavior is not changed.

Implementation of notifications will be problematic since there are two ways of achieving that:
- Passive way: that there will have to be some kind of method to keep connection to the services. If we consider big deployments, for example running hundreds of services that have to be configured, then the configuration service will have to keep connection to all of them. Either streaming (hanging connections) or reversing client/server roles can be used to achieve that.
  - First option is to use a mechanism that allows client or server to notify the other side that more data is on the way, however the problem is that it cannot be held indefinitely. Additional code will have to be put to provide proper medium and its behavior, and it will have to be tested separately.
  - Second option means that Server will act as a client and Client will act as a server. Configuration service will send the notification to servers that will look like a simple request with all necessary changes. This method, however, requires us to implement some additional code to register the service first, and a lot of additional work with securing the channels.

- Active way: each client can periodically check for updates, however this method will introduce unnecessary load and constantly lag behind the updates. The difference between config update and config propagation will be relative to the refresh time of the services. Additionally, it can prove problematic in case configuration have to be synchronized between services before work can be resumed.

Positives:
- initially easy to implement
- initially simple
- multiple implementation options and paths
- potentially mutable

neutral:
- some of the negatives mentioned below will disappear if features like dynamic configuration is to be cut out entirely, but it also invalidates this implementation completely since then, a static HTTP server can be used with exponentially lower implementation and maintenance costs
- there are a lot of problems that we can run into depending on the method or tool chosen. Also having a lot of leeway in choosing the method may cause later full rewrites of the feature as we learn more about the solution in question.
- potentially synchronization problems if synchronization is there to be implemented (multiple instances)

negatives:
- implementation cost rises significantly with complexity (number of features)
- have to be thoroughly tested (both network and behavior layer)
- notifications are problematic
- heavy load on the server in case of large deployments
- in case multiple configuration services are to be used, a replication and consent algorithms have to be implemented

Summary:
- relative work: 5/10
- maintenance work (periodic): 2/10
- flexibility of the solution: 5/10
- available features: 5/10
- work needed to expand in the future: 5/10

### P2P network
In this section, libp2p was taken into consideration as way-to-go library. P2P network can alleviate most of the programmer-side issues that are explained in previous section, while providing transparent that is scaling with the deployment itself. Libp2p works by providing the implementor with a set of default "network behaviors" that dictates how the node should behave in various situations, examples of which being particularly important to the subject at hand are "gossipsub" and "request-response". Explanation of those behaviors are below:
- gossipsub - more advanced publisher subscriber model (pub-sub being a best suited model for any p2p and/or meshing networks). One limitation of this model is the requirement to have at least 3 services running. General characteristics of the behavior is :
  - Services are building the network automatically and keeping only few connections to nearby peers, building a "mesh".
  - Requests in that case are being broadcasted onto the mesh until all peers receive the message.
  -  Messages are unique, do not collide, and cannot be duplicated.
  - Gossip sub protocol guarantees that each mesh member will get the message delivered at least once, however getting multiple messages with the same id will not generate new events as the message is already known.
This model does not depend on a centralized server to handle all the connections itself (connections per node, also server node, are quite limited, although configurable). That means even in large deployments, there is no need to keep multiple configuration services, but the model also do not prevent to have multiple configuration service instances per one deployment (i.e. replicas). This behavior is mainly used to send broadcast notifications to all the nodes listening on a particular topic.
In case replication is required, easy implementation using this behavior can be provided almost with very low effort.
- request-response - it's a simple protocol which will have to be defined (only generic implementation is available) that behaves like a standard request-response pattern communication. Usage of this functionality requires from the client side to know what is the target identity, which is a structure that holds the actual peer's identity and its routing information, like address and port or a path to unix socket. This information can be obtained and stored during the connecting to the network itself, autodetection of the services, or during receiving broadcast message. Downside of this method is that direct connection have to be established to the configuration service, and this can generate quite a load in specific situations.

To sum up, the library provides a lot of development glue that is not visible by the developer, and which is already tested in the library itself. It provides an abstract layer of communication that requires little to no additional preparation. The downside of this method is that we will have to provide a utility library with both client and server parts, and each service that needs to be autoconfigured, will have to use the client create (or for different languages, develop their own using other libp2p implementations written in target or target compatible language)

Positives:
- most of the features are provided out of the box
- notifications are working nicely
- best of two worlds - combining request-response and publisher-subscriber
- security out of the box
- upcoming features will be fairly easy to implement
- low maintenance, done right at the start it will not require too much work later
- p2p is transparent for the user and devs
- network autoconfiguration and autodetection of services out of the box

neutral:
- there is more work to be done to achieve reasonable degree of authorization
- libp2p implementations are available for major languages, but not all of them

negatives:
- lot of work initially
- there is a possibility that replication will have to be implemented if we want to keep multiple configuration services in sync

Summary:
- relative work: 7/10
- maintenance work (periodic): 2/10
- flexibility of the solution: 6/10
- available features: 8/10
- work needed to expand in the future: 3/10

### Addendum

#### libP2P
Both p2p request-response and general rest's major problems is related to the peak load. Those issues can be alleviated in relatively similar way, both client-side and server side. Examples of which are, implementation of random sleep periods while acting on the notifications, or having multiple configuration services available. Those options will not be explored in this document because those are tightly coupled with implementation itself.

While compatibility with other libp2p libraries written in different languages is stated in documentation and implied by following the same spec, it was not tested in the field by the author of this RFC.

Currently this is the state of the available language implementation
- C/C++ (working)
- java (in progress)
- python (in progress)
- rust (working)
- go (working)
- javascript (working)
- Nim (working)

In case those api are not enough, either additional endpoint (for example rest) will be provided or user-facing one shall be used (unprivilaged one).

#### Notifications
I would strongly recommend implementing notifications as a feature. This will lessen the strain on the configuration service, and allow for future expansion with more features that work on-demand and per-case, like what was mentioned in the introduction - High-Level Management.

#### Zookeeper
Other, alternative solutions were explored, mainly, as requested, zookeeper. However, this particular choice was not deemed to be the proper tool for providing configuration. Its usage would have to be coupled with already existing service, let's call it "configuration service" in this scope. Configuration service would have to provide or set configurations for each zookeeper node, diminishing its usage to being a literary communication medium. While some features would be preserved like service discovery, zookeeper would still behave like a simple KV database. As such it's just another KV database but with extra steps and little to none of sustantial gain for us.

## Chosen solution
I, personally, would opt out for libp2p solution, as the investment seems to be worth it. The amount of work required is quite high, however it can be treated as an investment, since after initial feature release is ready, almost no additional work have to be done. P2P part only have to be wrapped in common library. It is providing automatic, invisible layer to the users, and it does not have to be expanded further. All feature-related code is then common between the solution with the rest server. From my personal understanding, every other solution has to implement some if not all features that are already present in the libp2p either way, so libp2p that is offering most of those features out of the box (already implemented in protocol level) is really nice fit.

Some base work has already been done in separate [repo][magnetite], it can also be treated as a result of the research.

Additionally, not all features have to be implemented right now or at all, for example if we decide during development that mDNS (autodiscovery features) is not working properly for us It's okay to skip it.

#####  References
- [CDL project](https://github.com/epiphany-platform/CommonDataLayer)



[rfc2119]: https://www.ietf.org/rfc/rfc2119.txt
[magnetite]: https://github.com/erglabs/magnetite
