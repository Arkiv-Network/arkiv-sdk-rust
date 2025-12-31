# Arkiv SDK Examples

This directory contains minimal examples for using the `arkiv_sdk` library, which builds upon `alloy`.
`alloy` has its own community maintained [examples repository](https://github.com/alloy-rs/examples), which has ample examples for interacting
with different providers, wallet types and much more.

The examples in this directory only touch on how to create an Ethereum based provider, since Arkiv is
just an extension of Ethereum to support arbitrary storage on-chain.

`alloy`'s approach is entirely dynamic trait based, and so the `arkiv_sdk` simply extends what `alloy`
is already doing and maintains both dynamic compatibility and WASM support. The module structure is
intentionally modeled after `alloy` in order to make discovery of both `alloy` types and `arkiv_sdk`
types easier.

These examples touch on two main topics: constructing a provider to interact with entities on the Arkiv
network and event tracking from the Arkiv solidity contract.

## Setup

`arkiv` currently doesn't have a straight-forward way of testing locally, like `anvil`. Instead,
you'll need to use the `golembase-op-geth` docker container and command line application to spin up
a local instance, wallet and fund an account. The following steps show how to manually do this.

> NOTE: You will need `docker` and a `go` toolchain in order to run the node and build the command line app.

1. clone the [golembase-op-geth repository](https://github.com/Golem-Base/golembase-op-geth), or [download a source release](https://github.com/Golem-Base/golembase-op-geth/releases)
2. [run the docker container](https://github.com/Golem-Base/golembase-op-geth/blob/main/RUN_LOCALLY.md) to spin up a local node
3. use the command line application to [create and add funds to an account](https://github.com/Golem-Base/golembase-op-geth/blob/main/cmd/golembase/README.md)


