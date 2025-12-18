# Arkiv SDK Examples

This directory contains minimal examples for using the `arkiv_sdk` library, which builds upon `alloy`.
`alloy` has its own community maintained [examples repository](), which has ample examples for interacting
with different providers, wallet types and much more.

The examples in this directory only touch on how to create an Ethereum based provider, since Arkiv is
just an extension of Ethereum to support arbitrary storage on-chain.

`alloy`'s approach is entirely dynamic trait based, and so the `arkiv_sdk` simply extends what `alloy`
is already doing and maintains both dynamic compatibility and WASM support. The module structure is
intentionally modeled after `alloy` in order to make discovery of both `alloy` types and `arkiv_sdk`
types easier.

These examples touch on two main topics: constructing a provider to interact with entities on the Arkiv
network and event tracking from the Arkiv solidity contract.
