# Dapp

This contract utilizes the [IDapp](./../../traits/dapp/mod.rs) trait, in addition to the `AccessControl` traits. It is a straightforward dapp that generates cross-chain requests with or without acknowledgement, and also accepts requests from other chains.

## Build

To create the gateway, execute the following command.

```sh
cargo contract build --release
```