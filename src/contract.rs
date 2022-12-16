#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Binary, Addr, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint64, Uint128, Timestamp,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, VaultResponse, QueryMsg, ReceiveMsg};
use crate::state::{Config, Vault, CONFIG, VAULTS, Ledger};
use cw20::{Cw20Contract, Cw20ExecuteMsg};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw20-vault";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = info.sender;

    let config = Config {
        owner: owner.clone(),
        cw20_addr: deps.api.addr_validate(msg.cw20_addr.as_str())?,
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", owner)
        .add_attribute("cw20_addr", msg.cw20_addr)
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateVault{} => {
            // check if vault exists
            match VAULTS.load(deps.storage, info.sender.clone()) {
                Ok(_) => return Ok(Response::new()
                    .add_attribute("method", "execute_create_vault")
                    .add_attribute("msg", "vault already exists")
                ),
                Err(_) => ()
            };

            let vault = Vault {
                owner_addr: Addr::unchecked(info.sender.clone()),
                collected: Uint128::new(0),
                ledger_list: vec![],
            };
            VAULTS.save(deps.storage, info.sender.clone(), &vault)?;
            Ok(Response::new()
                .add_attribute("method", "execute_create_vault")
                .add_attribute("owner", info.sender.clone())
            )
        }
        ExecuteMsg::Withdraw { amount } => {
            let mut vault = VAULTS.load(deps.storage, info.sender.clone())?;
            if amount > vault.collected {
                return get_withdraw_fail_msg(vault.collected);
            }
            let ledgers = &mut vault.ledger_list;
            let cur_time = env.block.time.seconds();
            let mut amount_sum = Uint128::new(0);

            // iterate and withdraw coin from ledger
            for l in ledgers.iter_mut() {
                // withdraw within 1 minute of deposit fails
                if cur_time - l.receive_time.seconds() <= 60 {
                    break;
                }
                if amount_sum + l.coin_amount >= amount {
                    l.coin_amount -= amount - amount_sum;
                    amount_sum = amount;
                    break;
                }
                amount_sum += l.coin_amount;
                l.coin_amount = Uint128::new(0);
            }

            if amount_sum != amount {
                return get_withdraw_fail_msg(vault.collected);
            }

            vault.collected -= amount;
            remove_empty_ledger(ledgers);

            // save updated vault
            VAULTS.save(deps.storage, info.sender.clone(), &vault)?;

            // send CW20 to user
            let config = CONFIG.load(deps.storage)?;
            let cw20 = Cw20Contract(config.cw20_addr);

            // Build a cw20 transfer send msg
            let msg = cw20.call(Cw20ExecuteMsg::Transfer {
                recipient: info.sender.clone().to_string(),
                amount: amount,
            })?;

            Ok(Response::new()
                .add_attribute("method", "execute_withdraw")
                .add_attribute("is_success", "true")
                .add_attribute("get_amount", amount)
                .add_attribute("remaining_amount", vault.collected)
                .add_message(msg)
            )
        }
        ExecuteMsg::Receive(msg) => {
            let config = CONFIG.load(deps.storage)?;
            // ExecuteMsg::Receive msg should be sent by cw20 contract
            if config.cw20_addr != info.sender {
                return Err(ContractError::Unauthorized {});
            }

            let receive_msg: ReceiveMsg = from_binary(&msg.msg)?;

            deposit_vault(deps, receive_msg.vault_owner_addr.clone(), msg.amount, env.block.time.nanos())?;

            Ok(Response::new()
                .add_attribute("method", "execute_receive")
                .add_attribute("amount", msg.amount)
                .add_attribute("timestamp", Uint64::new(env.block.time.nanos()))
            )
        }
    }
}

pub fn deposit_vault(deps: DepsMut, addr: Addr, amount: Uint128, timestamp: u64) -> StdResult<()> {
    let mut vault = VAULTS.load(deps.storage, addr.clone())?;
    vault.collected += amount;
    vault.ledger_list.push(Ledger { coin_amount: amount, receive_time: Timestamp::from_nanos(timestamp) });
    VAULTS.save(deps.storage, addr.clone(), &vault)
}

pub fn get_withdraw_fail_msg(collected: Uint128) -> Result<Response, ContractError> {
    return Ok(Response::new()
        .add_attribute("method", "execute_withdraw")
        .add_attribute("is_success", "false")
        .add_attribute("get_amount", Uint128::new(0))
        .add_attribute("remaining_amount", collected)
    )
}

pub fn remove_empty_ledger(ledgers: &mut Vec<Ledger>) {
    ledgers
        .retain(|l| l.coin_amount > Uint128::new(0))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetVault { vault_owner_addr: vault_admin_addr } => to_binary(&query_vault(deps, vault_admin_addr)?),
    }
}

fn query_vault(deps: Deps, addr: Addr) -> StdResult<VaultResponse> {
    let vault = VAULTS.load(deps.storage, addr)?;
    Ok(VaultResponse {
        owner_addr: vault.owner_addr.to_string(),
        collected: vault.collected,
        ledger_list: vault.ledger_list,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{from_binary, Addr, CosmosMsg, WasmMsg};
    use cw20::Cw20ReceiveMsg;

    #[test]
    fn create_vault() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
        };
        let info = mock_info("tx_sender", &[]);

        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // create 1st vault
        let msg = ExecuteMsg::CreateVault{};
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes.get(1).unwrap().value, "tx_sender");

        let msg = QueryMsg::GetVault { vault_owner_addr: info.sender.clone() };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();

        let vault: Vault = from_binary(&res).unwrap();
        assert_eq!(
            vault,
            Vault {
                owner_addr: Addr::unchecked("tx_sender"),
                collected: Uint128::new(0),
                ledger_list: vec![],
            }
        );

        // create 2nd vault
        let msg = ExecuteMsg::CreateVault{};
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(res, Response::new()
            .add_attribute("method", "execute_create_vault")
            .add_attribute("msg", "vault already exists")
        );
    }

    #[test]
    fn receive() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
        };
        let info = mock_info("tx_sender", &[]);

        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // create vault
        let msg = ExecuteMsg::CreateVault{};
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes.get(1).unwrap().value, "tx_sender");

        // receive Cw20ReceiveMsg from cw20 contract
        let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: String::from(MOCK_CONTRACT_ADDR),
            amount: Uint128::new(100),
            msg: to_binary(&ReceiveMsg { vault_owner_addr: Addr::unchecked("tx_sender") }).unwrap(),
        });
        let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(120);
        let res = execute(deps.as_mut(), env, info.clone(), msg).unwrap();
        assert_eq!(res, Response::new()
            .add_attribute("method", "execute_receive")
            .add_attribute("amount", Uint128::new(100))
            .add_attribute("timestamp", Uint64::new(Timestamp::from_seconds(120).nanos()))
        );

        // query vault
        let msg = QueryMsg::GetVault { vault_owner_addr: Addr::unchecked("tx_sender") };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();

        let vault: Vault = from_binary(&res).unwrap();
        assert_eq!(
            vault,
            Vault {
                owner_addr: Addr::unchecked("tx_sender"),
                collected: Uint128::new(100),
                ledger_list: vec![Ledger { coin_amount: Uint128::new(100), receive_time: Timestamp::from_seconds(120) }],
            }
        );
    }

    #[test]
    fn withdraw() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
        };
        let info = mock_info("tx_sender", &[]);

        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(0);

        let _res = instantiate(deps.as_mut(), env, info.clone(), msg).unwrap();

        // create vault after 1s
        let msg = ExecuteMsg::CreateVault{};
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(1);

        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes.get(1).unwrap().value, "tx_sender");

        // Deposit 100 coin after 1 min of vault creation
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(60);
        let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: String::from(MOCK_CONTRACT_ADDR),
            amount: Uint128::new(100),
            msg: to_binary(&ReceiveMsg { vault_owner_addr: Addr::unchecked("tx_sender") }).unwrap(),
        });
        let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
        let res = execute(deps.as_mut(), env, info.clone(), msg).unwrap();
        assert_eq!(res, Response::new()
            .add_attribute("method", "execute_receive")
            .add_attribute("amount", Uint128::new(100))
            .add_attribute("timestamp", Uint64::new(Timestamp::from_seconds(60).nanos()))
        );

        // query vault
        let msg = QueryMsg::GetVault { vault_owner_addr: Addr::unchecked("tx_sender") };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();

        let vault: Vault = from_binary(&res).unwrap();
        assert_eq!(
            vault,
            Vault {
                owner_addr: Addr::unchecked("tx_sender"),
                collected: Uint128::new(100),
                ledger_list: vec![Ledger { coin_amount: Uint128::new(100), receive_time: Timestamp::from_seconds(60) }],
            }
        );

        // withdraw in less than 1 minute fails
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(90);
        let msg = ExecuteMsg::Withdraw { amount: Uint128::new(50) };
        let info = mock_info("tx_sender", &[]);
        let res = execute(deps.as_mut(), env, info.clone(), msg).unwrap();
        assert_eq!(res.attributes.get(1).unwrap().value, "false");

        // query vault
        let msg = QueryMsg::GetVault { vault_owner_addr: Addr::unchecked("tx_sender") };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();

        let vault: Vault = from_binary(&res).unwrap();
        assert_eq!(
            vault,
            Vault {
                owner_addr: Addr::unchecked("tx_sender"),
                collected: Uint128::new(100),
                ledger_list: vec![Ledger { coin_amount: Uint128::new(100), receive_time: Timestamp::from_seconds(60) }],
            }
        );

        // withdraw more than reserved coins fails
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(121);
        let msg = ExecuteMsg::Withdraw { amount: Uint128::new(150) };
        let info = mock_info("tx_sender", &[]);
        let res = execute(deps.as_mut(), env, info.clone(), msg).unwrap();
        assert_eq!(res.attributes.get(1).unwrap().value, "false");

        // query vault
        let msg = QueryMsg::GetVault { vault_owner_addr: Addr::unchecked("tx_sender") };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();

        let vault: Vault = from_binary(&res).unwrap();
        assert_eq!(
            vault,
            Vault {
                owner_addr: Addr::unchecked("tx_sender"),
                collected: Uint128::new(100),
                ledger_list: vec![Ledger { coin_amount: Uint128::new(100), receive_time: Timestamp::from_seconds(60) }],
            }
        );

        // withdraw 50 coins
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(121);
        let msg = ExecuteMsg::Withdraw { amount: Uint128::new(50) };
        let info = mock_info("tx_sender", &[]);
        let res = execute(deps.as_mut(), env, info.clone(), msg).unwrap();
        let cosmo_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: String::from(MOCK_CONTRACT_ADDR),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: String::from("tx_sender"),
                amount: Uint128::new(50)
            }).unwrap(),
            funds: vec![]
        });
        assert_eq!(res, Response::new()
            .add_attribute("method", "execute_withdraw")
            .add_attribute("is_success", "true")
            .add_attribute("get_amount", Uint128::new(50))
            .add_attribute("remaining_amount", Uint128::new(50))
            .add_message(cosmo_msg)
        );

        // query vault
        let msg = QueryMsg::GetVault { vault_owner_addr: Addr::unchecked("tx_sender") };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();

        let vault: Vault = from_binary(&res).unwrap();
        assert_eq!(
            vault,
            Vault {
                owner_addr: Addr::unchecked("tx_sender"),
                collected: Uint128::new(50),
                ledger_list: vec![Ledger { coin_amount: Uint128::new(50), receive_time: Timestamp::from_seconds(60) }],
            }
        );

        // Deposit 100 coin more
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(150);
        let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: String::from(MOCK_CONTRACT_ADDR),
            amount: Uint128::new(100),
            msg: to_binary(&ReceiveMsg { vault_owner_addr: Addr::unchecked("tx_sender") }).unwrap(),
        });
        let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
        let res = execute(deps.as_mut(), env, info.clone(), msg).unwrap();
        assert_eq!(res, Response::new()
            .add_attribute("method", "execute_receive")
            .add_attribute("amount", Uint128::new(100))
            .add_attribute("timestamp", Uint64::new(Timestamp::from_seconds(150).nanos()))
        );

        // query vault
        let msg = QueryMsg::GetVault { vault_owner_addr: Addr::unchecked("tx_sender") };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();

        let vault: Vault = from_binary(&res).unwrap();
        assert_eq!(
            vault,
            Vault {
                owner_addr: Addr::unchecked("tx_sender"),
                collected: Uint128::new(150),
                ledger_list: vec![
                    Ledger { coin_amount: Uint128::new(50), receive_time: Timestamp::from_seconds(60) },
                    Ledger { coin_amount: Uint128::new(100), receive_time: Timestamp::from_seconds(150) },
                ],
            }
        );

        // Try withdrawing 100 coins but fail. It can receive 50 coins but should wait for other 50 coins
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(200);
        let msg = ExecuteMsg::Withdraw { amount: Uint128::new(100) };
        let info = mock_info("tx_sender", &[]);
        let res = execute(deps.as_mut(), env, info.clone(), msg).unwrap();
        assert_eq!(res.attributes.get(1).unwrap().value, "false");

        // query vault
        let msg = QueryMsg::GetVault { vault_owner_addr: Addr::unchecked("tx_sender") };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();

        let vault: Vault = from_binary(&res).unwrap();
        assert_eq!(
            vault,
            Vault {
                owner_addr: Addr::unchecked("tx_sender"),
                collected: Uint128::new(150),
                ledger_list: vec![
                    Ledger { coin_amount: Uint128::new(50), receive_time: Timestamp::from_seconds(60) },
                    Ledger { coin_amount: Uint128::new(100), receive_time: Timestamp::from_seconds(150) },
                ],
            }
        );

        // Deposit 100 coin more
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(220);
        let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: String::from(MOCK_CONTRACT_ADDR),
            amount: Uint128::new(100),
            msg: to_binary(&ReceiveMsg { vault_owner_addr: Addr::unchecked("tx_sender") }).unwrap(),
        });
        let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
        let res = execute(deps.as_mut(), env, info.clone(), msg).unwrap();
        assert_eq!(res, Response::new()
            .add_attribute("method", "execute_receive")
            .add_attribute("amount", Uint128::new(100))
            .add_attribute("timestamp", Uint64::new(Timestamp::from_seconds(220).nanos()))
        );

        // query vault
        let msg = QueryMsg::GetVault { vault_owner_addr: Addr::unchecked("tx_sender") };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();

        let vault: Vault = from_binary(&res).unwrap();
        assert_eq!(
            vault,
            Vault {
                owner_addr: Addr::unchecked("tx_sender"),
                collected: Uint128::new(250),
                ledger_list: vec![
                    Ledger { coin_amount: Uint128::new(50), receive_time: Timestamp::from_seconds(60) },
                    Ledger { coin_amount: Uint128::new(100), receive_time: Timestamp::from_seconds(150) },
                    Ledger { coin_amount: Uint128::new(100), receive_time: Timestamp::from_seconds(220) },
                ],
            }
        );

        // Withdraw 200 coins. Total withdraw coins are 250
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(300);
        let msg = ExecuteMsg::Withdraw { amount: Uint128::new(200) };
        let info = mock_info("tx_sender", &[]);
        let res = execute(deps.as_mut(), env, info.clone(), msg).unwrap();
        let cosmo_msg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: String::from(MOCK_CONTRACT_ADDR),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: String::from("tx_sender"),
                amount: Uint128::new(200)
            }).unwrap(),
            funds: vec![]
        });
        assert_eq!(res, Response::new()
            .add_attribute("method", "execute_withdraw")
            .add_attribute("is_success", "true")
            .add_attribute("get_amount", Uint128::new(200))
            .add_attribute("remaining_amount", Uint128::new(50))
            .add_message(cosmo_msg)
        );

        // query vault
        let msg = QueryMsg::GetVault { vault_owner_addr: Addr::unchecked("tx_sender") };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();

        let vault: Vault = from_binary(&res).unwrap();
        assert_eq!(
            vault,
            Vault {
                owner_addr: Addr::unchecked("tx_sender"),
                collected: Uint128::new(50),
                ledger_list: vec![Ledger { coin_amount: Uint128::new(50), receive_time: Timestamp::from_seconds(220) }],
            }
        );
    }
}
