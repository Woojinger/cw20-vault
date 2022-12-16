# CW20 Vault Contract
## What does this contract do?
User can create vault which stores PGCoin(CW20).\
User can deposit their PGCoin(CW20) to vault.
- All coin deposits are recorded in amount and deposit time
```Shell
# when you query vault info
data:
  # all coins vault have
  collected: "300"
  # all deposits recoreded when you despoits coins to vault
  # receive time is UNIX time nanos
  ledger_list:
  - coin_amount: "100"
    # 22.12.16 08:04:16 GMT+00:00
    receive_time: "1671177856257807410"
  - coin_amount: "200"
    # 22.12.16 08:21.33 GMT+00:00
    receive_time: "1671178893141157818"
  # vault owner
  owner_addr: juno1sulm4ga8jgd73zs5q9wsumszu7ns6nkgxxvf3l
```
User can withdraw their PGCoin from vault in some condition.\
Withdrawing can be executed after 1 minute of coin deposit. \
coins to be withdrawn are determined in order of deposit.
- If you try to withdraw 200 coin from above vault at 22.12.16 08:06:00, it will fail
- If you try to withdraw 100 coin from above vault at 22.12.16 08:06:00, it will success
- If you try to withdraw 300 coin from above vault at 22.12.16 08:30:00, it will success
- If you try to withdraw 500 coin from above vault at 22.12.16 08:30:00, it will fail

# CW20 Vault Contract Info
network: Juno test network(uni-5) \
code_id: 3420 \
address: juno1cx3rj8qpxtzd8efqgjfxd2xjq6d0j5te3y7amzurp8upgwnyk43q44zljq

# CW20 Contract(PGCoin) Info
network: Juno test network(uni-5)\
address: juno1ka5p7mm8rfat7zs89xeegxyu9kxtljszckkdqfgv4e5x023c3hws7wjxaw\
name: PGCoin\
symbol: PGCoin\
total_supply: 1000000000000\
decimals: 6

# How to check Contract Operation
## 1. install junod and set testnet config

**you can modify contract command in sourceme.sh**

```Shell
source sourceme.sh
```
After you source the above variables, you should set the variables to junod
```Shell
junod config chain-id $CHAIN_ID
junod config node $RPC
```

## 2. Query CW20 Contract Info
```Shell
# Get balance info for juno1sulm4ga8jgd73zs5q9wsumszu7ns6nkgxxvf3l
junod query wasm contract-state smart $COIN_CONTRACT "$QUERY_OWNER_BALANCE" $NODE
# Get Token Info
junod query wasm contract-state smart $COIN_CONTRACT "$QUERY_TOKEN_INFO" $NODE
```

## 3. Create vault in cw20-vault Contract
### Create Vault of {account}
```Shell
junod tx wasm execute $VAULT_CONTRACT $CREATE $TXFLAG --from {account}  
```

### Send cw20 token to cw20-vault
```Shell
# Send 200 cw20 token to "juno1sulm4ga8jgd73zs5q9wsumszu7ns6nkgxxvf3l" vault
junod tx wasm execute $COIN_CONTRACT $SEND_TO_VAULT $TXFLAG --from testAccount1
```
**If you want to modify msg, you should create binary message like below**
```Rust
// If cw20-vault contract receive cw20 token, cw20-vault contract transfer cw20 token to "juno1sulm4ga8jgd73zs5q9wsumszu7ns6nkgxxvf3l" vault
let binmsg = to_binary(&ReceiveMsg { vault_owner_addr: Addr::unchecked("juno1sulm4ga8jgd73zs5q9wsumszu7ns6nkgxxvf3l") }).unwrap();
println!("{}", binmsg);
```

### Withdraw cw20 token from sender vault
```Shell
junod tx wasm execute $VAULT_CONTRACT $WITHDRAW $TXFLAG --from testAccount1
```

### Query vault
```Shell
# Get Vault of juno1sulm4ga8jgd73zs5q9wsumszu7ns6nkgxxvf3l
junod query wasm contract-state smart $CONTRACT $QUERY_VAULT $NODE
```

# Unit test, Compiling
Unit test
```Shell
cargo unit-test
```
Compiling
```Shell
cargo wasm
```

Optimize for **arm-64**
```
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer-arm64:0.12.8
```
# Other Resource
Please refer to the link below for help

[Cosmwasm 배포 및 실행하기](https://pangyoalto.com/cosmwasm-contract-2/)