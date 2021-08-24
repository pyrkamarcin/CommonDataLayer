# Front Matter

```
    Title           : Custom Data Passthrough
    Author(s)       : Mateusz 'esavier' Matejuk
    Team            : CommonDataLayer
    Reviewer        : CommonDataLayer
    Created         : 2021-08-10
    Last updated    : 2021-08-23
    Category        : Feature
    CDL Feature ID  : CDLF-00021-00
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
* `gRPC` - Remote procedure call protocol - [website][gRPC]
* `AMQP` - Advanced Message Queuing Protocol - [website][AMQP]
* `Kafka` - Distributed event streaming platform - [website][Kafka]
* `ingestion`/`ingest` - push new data into CDL system

## Features
[CDLF-00004-00](../features/index.md) - CDL Feature - Transport Header Passthrough

## RFCs
[0018 Transport Header Passthrough](./0018_Transport_headers_passthrough_01.md)

## Introduction
This is a new requirement for the CDL: support for passing through arbitrary data in the CDL system without acting on it. This RFC is related to RFC-0018, however it does not auto-convert headers, but accepts a custom field with arbitrary data.

This feature is being added to allow compatibility with systems that require metadata coordinated with data entering the CDL. Specifically, when that metadata is not needed by the CDL, but further down the pipeline.

## Formats
The only format that will change is the ingestion format.

Current Format:
```
   {
      version: String
      object_id: UUID,
      schema_id: UUID,
      data: { any valid json },
   }
```

New Format:
```
   {
      "objectId": "...",
      "schemaId": "...",
      "options": {
         passthrough: {
            "data": {...}
         },
      }
      "data": {}
   }
```

## Solution
Currently, we do not expect any conflicts with internal formats, as the new field is purely additive.
This field should be treated as fully optional. Data inside the `options.passthrough` object should not be acted upon in any capacity, including storage or usage.

The provided fields have to go through the system and be emitted by the Command Service's notification system. Since this data is not used, it should not be provided for notification methods in materialization or anywhere further than the data storage layer, excluding the database itself.

The notification format is not yet defined, and as such there is no way to describe what the output should be. Therefore, it is left for the implementer to decide.
If a later RFC or documentation defines a format, the notification format will have to be changed then.

### Testing
This feature MUST undergo thorough testing:
* Performance tests
* Green path
* General API tests


[gRPC]: https://grpc.io/
[gRPC-Protocol]: https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md
[Kafka]: https://kafka.apache.org/
[AMQP]: https://www.rabbitmq.com/amqp-0-9-1-reference.html

[rfc2119]: https://www.ietf.org/rfc/rfc2119.txt
