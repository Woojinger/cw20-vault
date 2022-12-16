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

export CODE_ID="3420"

export VAULT_CONTRACT="juno1cx3rj8qpxtzd8efqgjfxd2xjq6d0j5te3y7amzurp8upgwnyk43q44zljq"
export INIT='{"cw20_addr":"juno1ka5p7mm8rfat7zs89xeegxyu9kxtljszckkdqfgv4e5x023c3hws7wjxaw"}'
export CREATE_VAULT='{"create_vault":[]}'
export WITHDRAW='{"withdraw":{"amount":"300"}}'
export QUERY_VAULT='{"get_vault":{"vault_owner_addr":"juno1sulm4ga8jgd73zs5q9wsumszu7ns6nkgxxvf3l"}}'

export COIN_CONTRACT="juno1ka5p7mm8rfat7zs89xeegxyu9kxtljszckkdqfgv4e5x023c3hws7wjxaw"
export SEND_TO_VAULT='{"send":{"contract":"juno1cx3rj8qpxtzd8efqgjfxd2xjq6d0j5te3y7amzurp8upgwnyk43q44zljq", "amount":"200", "msg":"eyJ2YXVsdF9vd25lcl9hZGRyIjoianVubzFzdWxtNGdhOGpnZDczenM1cTl3c3Vtc3p1N25zNm5rZ3h4dmYzbCJ9"}}'
export QUERY_OWNER_BALANCE='{"balance":{"address":"juno1sulm4ga8jgd73zs5q9wsumszu7ns6nkgxxvf3l"}}'
export QUERY_TOKEN_INFO='{"token_info":{}}'
