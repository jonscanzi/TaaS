# Using the cloud to improve automated testing ![build status](https://github.com/jonscanzi/TaaS/workflows/build/badge.svg)

TaaS stands for __Testing as a Service__. This project aims to offer some features that can be expected from software testing on the cloud. The main focus of this project is to offer infrastructure-scale testing that require multiple machines to be run. TaaS's main features include:


* Can handle creation and handling of up to at least 20 machines (more is possible but not guaranteed)
* Abstraction of network interfaces
* Packaged into a single executable with configuration files
* YAML-based description of the machines that need to be created for a given test
* Support for Azure (using Azure CLI)
* Cloud provider interface to simplify developement for additional cloud providers

## Usage

Under the Release section, download links are provided for archives containing the orchestrator binary alongside a sample configuration (which needs to be modified) and a few examples to get started.

More documentation on how to run the demos will be added later.

### Code
The orchestrator is built in Rust and only uses public libraries from Rust crates. This means that to build the orchestrator, only **rustc** and **cargo** need to be installed in the system. These can be installed e.g. with rustup. Once setup, building the project only requires going into the **orchestrator** folder and running **cargo build**.  


