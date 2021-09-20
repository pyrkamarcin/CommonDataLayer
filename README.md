# This Project is Suspended
The decision to suspend the project was taken on 2021-09, and therefore we halted all work related to it.

Latest released verion is 1.1, however this is still open source, so it can be resurrected later or forked at any time. The team will try to track the community-generated issues that will be added to the official repository and resolve those if possible.

# Common Data Layer
A collection of microservices that make up the Common Data Layer, implemented in the [Rust][rust] language.

# What is Common Data Layer?
Common Data Layer (in short `cdl`) is multipurpose and multi-format, cloud native, abstract database application. It
enables user to store, read and materialize `documents`, `timeseries data` and `blobs` without the need to worry about
individual database APIs.

# Documentation
See [book][book] for more information how to use it and deploy.

# Quickstart
Read [configuration README](./.cdl/README.md)
and [configuration docs](https://epiphany-platform.github.io/CommonDataLayer/configuration/index.html).

CDL can be ran on k8s, on docker or on bare metal.

## k8s
Please refer to [documentation](https://epiphany-platform.github.io/CommonDataLayer/deployment/local/helm.html).
> Note that currently helm does not use `.cdl` directory for configuration

## docker
CDL images are available on [dockerhub](https://hub.docker.com/u/epiphanyplatform).  
Configuration can be added via docker volumes. Please keep in mind that it is likely that you'll have to review domain
names and ports.

## bare metal
CDL does not provide binaries for bare metal deployments. Binaries can be built manually via command:

```bash
cargo build --workspace
```

> As bare metal isn't really documented (and supported, however it's there for local testing), in case of problems feel free to open a discussion on our github repo

[rust]: https://www.rust-lang.org

[book]: https://epiphany-platform.github.io/CommonDataLayer/
