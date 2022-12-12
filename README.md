## CW20 Vault Contract
### What does this contract do?
- User can create vault which stores PGCoin(CW20)
- User can deposit their PGCoin(CW20) to vault
- User can withdraw their PGCoin from vault
  - Withdrawing can be executed after 1 minute of coin deposit
  - All coin deposits are recorded in amount and deposit time

### CW20 Vault Contract Info
network: Juno test network(uni-5)

## CW20 Contract Info
network: Juno test network(uni-5)\
address: juno1ka5p7mm8rfat7zs89xeegxyu9kxtljszckkdqfgv4e5x023c3hws7wjxaw\
name: PGCoin\
symbol: PGCoin\
total_supply: 1000000000000\
decimals: 6

## How to check Contract Operation
### 1. install junod and set testnet config
```Shell
export CHAIN_ID="uni-5"
export TESTNET_NAME="uni-5"
export DENOM="ujunox"
export BECH32_HRP="juno"
export WASMD_VERSION="0.27"
export JUNOD_VERSION="v9.0.0"
export CONFIG_DIR=".juno"
export BINARY="junod"

export COSMJS_VERSION="v0.28.4"
export GENESIS_URL="https://raw.githubusercontent.com/CosmosContracts/testnets/main/uni-5/genesis.json"
export PERSISTENT_PEERS_URL="https://raw.githubusercontent.com/CosmosContracts/testnets/main/uni-5/persistent_peers.txt"
export SEEDS_URL="https://raw.githubusercontent.com/CosmosContracts/testnets/main/uni-5/seeds.txt"

export RPC="https://rpc.uni.juno.deuslabs.fi:443"
export LCD="https://lcd.uni.juno.deuslabs.fi"
export FAUCET="https://faucet.uni.juno.deuslabs.fi"

export COSMOVISOR_VERSION="v0.1.0"
export COSMOVISOR_HOME=$HOME/.juno
export COSMOVISOR_NAME=junod

export NODE=(--node $RPC)
export TXFLAG=($NODE --chain-id $CHAIN_ID --gas-prices 0.025ujunox --gas auto --gas-adjustment 1.3 --broadcast-mode block)
```
After you source the above variables, you should set the variables to junod
```Shell
junod config chain-id $CHAIN_ID
junod config node $RPC
```

### 2. Set CW20 Contract Info and Query Command
```Shell
export COIN_CONTRACT=juno1ka5p7mm8rfat7zs89xeegxyu9kxtljszckkdqfgv4e5x023c3hws7wjxaw
export QueryTokenInfo='{"token_info":{}}'
# get balance for juno1sulm4ga8jgd73zs5q9wsumszu7ns6nkgxxvf3l
export QueryBalance='{"balance":{"address":"juno1sulm4ga8jgd73zs5q9wsumszu7ns6nkgxxvf3l"}}'
```
After source the above variables, you can query info from contract.

```Shell
# Get balance info for juno1sulm4ga8jgd73zs5q9wsumszu7ns6nkgxxvf3l
junod query wasm contract-state smart $COIN_CONTRACT "$QueryBalance" $NODE
# Get Token Info
junod query wasm contract-state smart $COIN_CONTRACT "$QueryTokenInfo" $NODE
```

### 3. Set CW20-Vault Contract Info and Query Command


Please refer to the link below for help

[Cosmwasm 배포 및 실행하기](https://pangyoalto.com/cosmwasm-contract-2/)