# Arkiv SDK for Rust

This is part of the [Arkiv](https://github.com/Arkiv-Network) project, which is designed as a Layer2 Network deployed on Ethereum, acting as a gateway to various Layer 3 Database Chains (DB-Chains).
For an overview of Arkiv, **check out our [Litepaper](https://arkiv.network/pdf/ARKIV_Litepaper_blue.pdf)**.

This SDK allows you to use [Arkiv](https://github.com/Golem-Base) from Rust, it is available on [crates.io](https://crates.io/crates/arkiv-sdk), alng with its [generated documentation](https://docs.rs/arkiv-sdk). We provide an [example application](https://github.com/Arkiv-Network/arkiv-sdk-rust/tree/main/demo) to showcase how you can use this SDK.

## Getting started

For **getting up and running quickly**, we recommend the following two steps:
1. Start golembase-op-geth through its [`docker-compose`](https://github.com/Golem-Base/golembase-op-geth/blob/main/RUN_LOCALLY.md) ;
2. [Install the demo CLI](https://github.com/Golem-Base/golembase-demo-cli?tab=readme-ov-file#installation) and [create a user](https://github.com/Golem-Base/golembase-demo-cli?tab=readme-ov-file#quickstart), or build the [actual CLI](https://github.com/Golem-Base/golembase-op-geth/blob/main/cmd/golembase/README.md) as it's included in the `golembase-op-geth` repository.

When you create a user, it will generate a new wallet file called `wallet.json` and store it in the standard folder as per the [XDG specification](https://specifications.freedesktop.org/basedir-spec/latest/):
- `~/.config/golembase/` on **Linux**
- `~/Library/Application Support/golembase/` on **macOS**
- `%LOCALAPPDATA%\golembase\` on **Windows**

You will also need to fund the account, you can do it with: `golembase-demo-cli account fund 10`

## Overview

This SDK provides multiple layers for sending transactions:
- Use `ArkivClient` for high-level operations such as creating, updating, or deleting entities.
- Use `Account` for account-centric and lower-level transaction control.
- Advanced users can construct and submit raw Ethereum transactions directly using the types and helpers re-exported from `Alloy`.

## Contributing

- Enter a reproducible [Nix flakes](https://wiki.nixos.org/wiki/Flakes) devshell with `nix develop` or use [`direnv`](https://direnv.net/)
- Install pre-commit git hook with `pre-commit install`

Thanks for helping improve the project!
