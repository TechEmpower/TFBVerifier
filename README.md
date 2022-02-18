# TFBVerifier

[![Build Status](https://github.com/TechEmpower/TFBVerifier/workflows/build/badge.svg?branch=master&event=push)](https://github.com/TechEmpower/TFBVerifier/actions?query=workflow%3Abuild+branch%3Amaster)

The application that verifies the response from an application running in the
[TechEmpower Framework Benchmarks](https://github.com/TechEmpower/FrameworkBenchmarks).

The goal of this application is to live in isolation from 
[test implementations](https://github.com/TechEmpower/FrameworkBenchmarks) and 
even the [TFBToolset](https://github.com/TechEmpower/TFBToolset). This 
application contains a Dockerfile which is how the Docker Image is created and 
eventually published to Dockerhub.

The TFBToolset uses that published Docker image to verify test implementations
in the FrameworkBenchmarks project.

## Getting Started

These instructions will get you a copy of the project up and running on your 
local machine for development and testing purposes.

### Prerequisites

* [Rust](https://rustup.rs/)
* [Docker](https://docs.docker.com/engine/install/)* or [Docker4Windows](https://docs.docker.com/docker-for-windows/install/)*
* [TechEmpower Frameworks](https://github.com/TechEmpower/FrameworkBenchmarks)*
* [TFBToolset](https://github.com/TechEmpower/TFBToolset)*

\* Not required for development or testing; only full-suite testing and deploying.

#### Windows Only

* [Expose daemon on `tcp://localhost:2375`](https://docs.docker.com/docker-for-windows/#general)*

\* Not required for development or testing; only full-suite testing and deploying.

### Running the tests

```
$ cargo test
```

### Building

```
$ cargo build --release
```

### Installing

```
$ docker build -t techempower/tfb.verifier .
```

## Running

To run any verification, a test implementation must be running from the 
TFBToolset in `debug` mode, which will attach the test implementation to the
Docker Network `TFBNetwork`.

```
$ docker run -it --network=TFBNetwork -e "MODE=[probably verify, but maybe benchmark]" -e "CONCURRENCY_LEVELS=16,32,64,128,256,512" -e "PIPELINE_CONCURRENCY_LEVELS=256,1024,4096,16384" -e "DATABASE=[database you want to verify; this env var *can* be ignored]" -e "PORT=[the exposed port]" -e "TEST_TYPE=[the test type you want to verify]" -e "ENDPOINT=[the relative URL]" tfb.verifier
```

## Authors

* **Mike Smith** - *Initial work* - [msmith](https://github.com/msmith-techempower)

## License

This project is licensed under the BSD-3-Clause License - see the [LICENSE.md](LICENSE.md) file for details
