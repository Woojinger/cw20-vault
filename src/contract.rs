#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, Binary, Addr, Deps, DepsMut, Env, MessageInfo, Response, StdResult, StdError,
    Uint64, Uint128, Timestamp,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, VaultResponse, QueryMsg, ReceiveMsg};
use crate::state::{Config, Vault, CONFIG, VAULTS, Ledger};
use cw20::{Cw20Contract, Cw20ReceiveMsg};

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
    // TODO
    // Temporary code for passing test
    match msg {
        ExecuteMsg::CreateVault() => {
            // check if vault exists
            match VAULTS.load(deps.storage, info.sender.clone()) {
                Ok(_) => return Ok(Response::new()
                    .add_attribute("method","execute_create_vault")
                    .add_attribute("msg","vault already exists")
                ),
                Err(_) => ()
            };

            let vault = Vault {
                admin_addr: Addr::unchecked(info.sender.clone()),
                collected: Uint128::new(0),
                ledger_list: vec![],
            };
            VAULTS.save(deps.storage, info.sender.clone(), &vault)?;
            Ok(Response::new()
                .add_attribute("method", "execute_create_vault")
                .add_attribute("owner",info.sender.clone())
            )
        }
        ExecuteMsg::Withdraw { vault_id: vault_id, amount: amount } => {
            // TODO
            Ok(Response::new())
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

pub fn deposit_vault(deps: DepsMut, addr: Addr, amount: Uint128, timestamp: u64) -> StdResult<()>{
    let mut vault = VAULTS.load(deps.storage, addr.clone())?;
    vault.collected += amount;
    vault.ledger_list.push(Ledger{coin_amount: amount, receive_time: Timestamp::from_nanos(timestamp) });
    VAULTS.save(deps.storage, addr.clone(), &vault)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetVault { vault_admin_addr } => to_binary(&query_vault(deps, vault_admin_addr)?),
    }
}

fn query_vault(deps: Deps, addr: Addr) -> StdResult<VaultResponse> {
    let vault = VAULTS.load(deps.storage,addr)?;
    Ok(VaultResponse {
        admin_addr: vault.admin_addr.to_string(),
        collected: vault.collected,
        ledger_list: vault.ledger_list,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{from_binary, Addr, CosmosMsg, WasmMsg};

    #[test]
    fn create_vault() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
        };
        let info = mock_info("tx_sender", &[]);

        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // create 1st vault
        let msg = ExecuteMsg::CreateVault();
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes.get(1).unwrap().value, "tx_sender");

        let msg = QueryMsg::GetVault { vault_admin_addr: info.sender.clone() };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();

        let vault: Vault = from_binary(&res).unwrap();
        assert_eq!(
            vault,
            Vault {
                admin_addr: Addr::unchecked("tx_sender"),
                collected: Uint128::new(0),
                ledger_list: vec![],
            }
        );

        // create 2nd vault
        let msg = ExecuteMsg::CreateVault();
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(res, Response::new()
            .add_attribute("method","execute_create_vault")
            .add_attribute("msg","vault already exists")
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
        let msg = ExecuteMsg::CreateVault();
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes.get(1).unwrap().value, "tx_sender");

        // receive Cw20ReceiveMsg from cw20 contract
        let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: String::from(MOCK_CONTRACT_ADDR),
            amount: Uint128::new(100),
            msg: to_binary(&ReceiveMsg{vault_owner_addr: Addr::unchecked("tx_sender")}).unwrap(),
        });
        let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let time_stamp_nano_str = res.attributes.get(2).unwrap().clone().value;
        let time_stamp = Timestamp::from_nanos(time_stamp_nano_str.parse::<u64>().unwrap());

        // query vault
        let msg = QueryMsg::GetVault { vault_admin_addr: Addr::unchecked("tx_sender") };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();

        let vault: Vault = from_binary(&res).unwrap();
        assert_eq!(
            vault,
            Vault {
                admin_addr: Addr::unchecked("tx_sender"),
                collected: Uint128::new(100),
                ledger_list: vec![Ledger{coin_amount:Uint128::new(100), receive_time:time_stamp}],
            }
        );
    }
}
