# Crosstalk Samples

## PING-PONG

Here, we will deploy a cross-chain ping pong smart contract built using the Router CrossTalk. It is a system where we can send a message from the source chain(EVM) to a destination chain(EVM) and receive back the acknowledgement from the destination chain on the source chain. So basically, we will send a ping to the destination chain and receive a pong back to the source chain. For that to work, kindly follow the below mentioned steps for deployment:

1. Compile your contracts by first adding `.env` file and run
   ```shell
   npx hardhat compile
   ```
2. After compilation, check if the gateway addresses and the fee payer address mentioned for the respective chains on which contracts have to be deployed are updated [here](./deployment/deployments.json).
3. We have already added a hardhat task for deployment of ping-pong contract [here](./tasks/deploy/PingPong.ts). Run
   ```shell
   npx hardhat
   ```
   and check if `TASK_DEPLOY_PINGPONG` is listed in the tasks list.
4. You just need to run the following command for respective chain to get your contracts deployed on that chain.
   ```shell
   npx hardhat TASK_DEPLOY_PINGPONG --network <network_name>
   ```
   For example:
   1. If you want to deploy your contract on Polygon mumbai, you just have to run:
   ```shell
   npx hardhat TASK_DEPLOY_PINGPONG --network mumbai
   ```
   2. If you want to deploy your contract on Avalanche Fuji, you just have to run:
   ```shell
   npx hardhat TASK_DEPLOY_PINGPONG --network fuji
   ```
   and likewise.
