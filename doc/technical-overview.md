# TaaS technical documentation

This document is meant to explain how TaaS’s code is structured, as well as what every part does. For using TaaS, please refer to the **how to use** documents.

## General look - the orchestrator
The TaaS project consists mainly of the **orchestrator**. This is the base tool which is able to create, configure, and execute tests on multiple machines in the cloud. The orchestrator by itself only runs user-provided scripts and code, but it does not care about specific types of machine or software as it is meant to be usable for any project. As such, users are then expected to include the orchestrator into an overarching framework which provides auto-genrated configuration for the orchestrator to run.  

The orchestrator takes a number of configuration options from the **config/** folder. It expects the scenario that needs to be run to be present in the **scenarios/** folder. Finally, it uses a webserver (in **webserver/**) to handle sending scripts and data to the machines.





## Code structure

### General structure

The code uses a file-based encapsulation approach that Rust allows. Essentially, every folder is the base of a module (or a namespace in C++ terms). Then, a file named **mod.rs** is present, which is the only file that the Rust compiler will (initially/automatically) look into. This file just contains the declaration of the other files in that module. For example, if we take the function ``log2_ceil(x: i32)`` in **utils/math.rs**, its Rust path would be ``(crate::)math::log2_ceil(x: i32)``. Inside the ``mod.rs`` file in **utils/**, you can see the line ``pub mod math;`` which exports the module in a manner that the functions and objects can be accessed outside the module. Note that very simple modules (usually less than 100 l.o.c.) will have their code directly in ``mod.rs``.

### Walkthrough of files
Disclaimer: this section is meant to give a general idea of how the code is laid out. Most of the time, actual lines of code and functions will be omitted in order to focus on the general behaviour and purpose of every file / module.

### ``asir/mod.rs``
Contains several abstractions for the machine representation. They are mostly self-explanatory, except the ``Os`` trait. This is an abstraction that allows to choose how the chosen OS should be used. In practice the ``common_os`` field is the most useful (and the only one used in examples), although it is cloud specific.  


### ``azuresir/``

#### ``emitter.rs``
This is the helper module that will take an complete Azure system (that can be created with the module ``translator``), and it will generate a list of shell commands that will create the input system with Azure CLI.

#### ``system.rs``
The definition of the Azure system is here, containing the list of machines, their parameters (OS, Size, password, etc.) as well as the physical NICs and their attached subnets (required for a manual deployment of a machine network).

#### ``translator.rs``
This file defines the function to translate a PASIR system into its equivalent Azure representation. This is moslty starightforward, with the exception of NICs which must be created as they are not part of the PASIR representation.

#### ``translator.rs``
This file defines the function to translate a PASIR system into its equivalent Azure representation. This is moslty starightforward, with the exception of NICs which must be created as they are not part of the PASIR representation.

#### ``utils`.rs``
This contains 2 functions that are used to find the right OS given a vague string. This feature is not 100% required if the user-supplied OS is exact (i.e. already cloud specific).  


### ``cloud_functions/``
This is the folder that is meant to contain most cloud-specific functions (currently only for Azure machines/Azure CLI). Please note that future cloud-specific functions don't necessarily need to be in this folder, it is merely a convenience encapsulation.

#### ``azure`.rs``
This is the toolbox that contains most helper function to create and manage VMs with Azure CLI. This mostly includes functions to determine the Azure VM size to select (if a cloud-specific machine was not provided), as well as implementations that will be used in the abstractions (such as running a provided script on a specified machine).  


### ``lasir/``

#### ``connections.rs``
This represents a set of logical connections (i.e. represented as a graph with some details and not a real network). This is essentially a nice wrapper around the way users require connections from the machines. The system technically supports asymetric connections (i.e. the connections is 1-way), but they are not implemented.

#### ``machines.rs``
It is just a thin wrapper around the user-supplied machine parameters. It is the same as a PASIR machine.

#### ``translator.rs``
This file takes care of taking the raw YAML representation, and outputs it into LASIR. It is mostly straightforward as LASIR is close the the YAML representation. It just has a couple of rules to replace missing information with default values.


### ``logger/``

#### ``mod.rs``
This file defines a macro that wraps Rust's ``println!`` into a special print that prepends the current time. It is used throughout the project to offer this feature without duplicating code.

### ``orchetsrator/``

#### ``mod.rs``
This file handles the orchestrator webserver, which is an extra machine from where the deployed machines recover files and scripts (so as to not send large files to multiple VMs in the cloud multiple times).
Note that currently the orchestrator is implemented only as a cloud-agnostic-defined VM. If you implement support for a new cloud provider without cloud-agnostic support, you will need to change that.  


### ``pasir/``

#### ``connections.rs``
This contains both the physical netork (i.e. IP addresses and subnets), as well as the helper functions to construct the network (mainly the subnet generation algorithm).

#### ``machines.rs``
It is just a thin wrapper around the user-supplied machine parameters. It is the same as a LASIR machine.

#### ``translator.rs``
The functions defined here deal with going from LASIR to PASIR, which meant copying the machine parameters, and calling the PASIR connections submodule to translate the network.  


### ``pipelines/``
This is the main part of the framework, where all the steps are defined and executed. The layout is relatively simple.

#### ``azure_cli.rs``
It defines the 3 cloud-specific functions that need to be implemented for any given cloud provider (here for Azure, using Azure CLI). The general description of these 3 functions are defined below.

#### ``mod.rs``
This file contains all the code with all the steps to execute in order to deploy a system and run the tests. All steps are done one at a time, with many helper functions provided. Normally this file does not need to be changed as it is implemented as a trait with 3 undefined functions that bust be implemented by cloud-specific versions (and these specifc versions are the one that should be run from the main function).

The 3 cloud specific functions that need to be implemented to support running from a different cloud provider are:

* ``create_system()``, which takes a PASIR system and deploys machines to the cloud accordingly (it can be seen as being quite similar to how LASIR is translated to PASIR, with the extra function of deploying the system once it has been translated).
* ``run_script()``, which simply runs the provided script to the chosen machine (as root).
* ``get_public_ip()``, which should return a globally routable string (either IP address or fully qualified DNS name) for the given machine.  


### ``post_deployment/``

#### ``mod.rs``
This defines the post_deployment feature, which takes the user-provided ``post_deployment.sh`` for a given scenario, replaces generic values (like passwords) and runs it. It is mostly a set of helper functions for the main run procedure.

### ``script_push``
Is an _undocumented_ feature that allows script to be pushed to multiple VMs at the same time. It is meant to be only used manually by users.

It takes temporary files that are generated when a deployment is made (``last_deployment_summary.yml`` and ``last_deployment_replacements.yml``), and uses these informations to run a specific script to all machines.

### ``shell_tools/``

#### ``mod.rs``
This convenience module offers many ways to run a shell command, as well as many options on what to do if the command fails. It is meant to allow to write almost normal-looking shell commands (i.e. as a single string).  


### ``utils/``

#### ``global_config.rs``
This is a big list of wrappers around the configuration files. They are implemented as lazy static references in order to make them available as global constants (that are initialised at run-time).

#### ``macros.rs``
This file contains a miscellaneous set of convenience macros that are used in the project.

#### ``math.rs``
As the name suggests, this defines a couple of math functions that are not provided by the standard library.

#### ``replace.rs``
This is the place that defines all features of the replacement engine, used e.g. to subsitute connections into routable hostnames.

#### ``roles.rs``
Roles contain many defitions to handle replacement for roles. Roles is a feature that allow multiple independent network to describe machines in the same way. This feature is complicated to use and users are recommended to simply ignore it.


#### ``run_parser.rs``
This file takes care of parsing the ``pipeline.run`` file of a scenario. This is the file that defines every script that every machine should run.

#### ``types.rs``
This module contains the defintion of special types, such as the CIDR IP representation used by the PASIR and network.  


### ``yamlsir/``

#### ``default.rs``
This is the module that takes care of giving default settings to missing values from the user provided system configuration.

#### ``mod.rs``
This file defines the structure that directly come from the YAMl system description file. It uses serde which is a generic Rust serialiser.


### ``/`` (root)

### ``main.rs``
This is the fiel that takes care of parsing the input arguments to decide what to run (run a scenario or delete the existing deployment). Currently it doesn't make use of cloud-provider parameters and assumes it is always Azure (as it is currently the only supported cloud provider).

#### ``path.rs``
This is a simple list of file path that are not directly configurable by a user of the compiled framework.