# To deploy to testnet

use command to deploy:

```
near deploy xyz.abc.testnet --initFunction new --initArgs '{"gateway":"gateway.abc.testnet"}' --wasmFile target/wasm32-unknown-unknown/release/contract.wasm
```

> Note: Change the addresses of deployment and gateway before deploying.
