#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Addr, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint64, Uint128, Timestamp,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, VaultResponse, QueryMsg, ReceiveMsg};
use crate::state::{save_vault, Config, Vault, CONFIG, VAULTS, VAULT_SEQ, Ledger};
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

    VAULT_SEQ.save(deps.storage, &0)?;

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
            let vault = Vault {
                admin_addr: Addr::unchecked(info.sender),
                collected: Uint128::new(0),
                ledger_list: vec![],
            };
            let new_vault_id = save_vault(deps, &vault).unwrap();
            Ok(Response::new()
                .add_attribute("method", "execute_create_vault")
                .add_attribute("vault_id", Uint64::new(new_vault_id))
            )
        }
        ExecuteMsg::Withdraw { vault_id: vault_id, amount: amount } => {
            // TODO
            Ok(Response::new())
        }
        ExecuteMsg::Receive(msg) => {
            // TODO
            // Temporary implementation
            let new_vault_id = 0;
            Ok(Response::new()
                .add_attribute("method", "execute_receive")
                .add_attribute("vault_id",Uint64::new(new_vault_id))
                .add_attribute("amount", msg.amount)
                .add_attribute("timestamp", Uint64::new(env.block.time.nanos()))
            )
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetVault { id } => to_binary(&query_vault(deps, id)?),
    }
}

fn query_vault(deps: Deps, id: Uint64) -> StdResult<VaultResponse> {
    let vault = VAULTS.load(deps.storage, id.u64())?;
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
        assert_eq!(res.attributes.get(1).unwrap().value, "1");

        let msg = QueryMsg::GetVault { id: Uint64::new(1) };
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
        assert_eq!(res.attributes.get(1).unwrap().value, "2");

        let msg = QueryMsg::GetVault { id: Uint64::new(2) };
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
    }

    #[test]
    fn receive() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            cw20_addr: String::from(MOCK_CONTRACT_ADDR),
        };
        let info = mock_info("tx_sender", &[]);

        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // create 1st vault
        let msg = ExecuteMsg::CreateVault();
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        assert_eq!(res.attributes.get(1).unwrap().value, "1");

        // receive Cw20ReceiveMsg from cw20 contract
        let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
            sender: String::from(MOCK_CONTRACT_ADDR),
            amount: Uint128::new(100),
            msg: to_binary(&ReceiveMsg::Send { vault_id: Uint64::new(1) }).unwrap(),
        });
        let info = mock_info(MOCK_CONTRACT_ADDR, &[]);
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let time_stamp_nano_str = res.attributes.get(3).unwrap().clone().value;
        let time_stamp = Timestamp::from_nanos(time_stamp_nano_str.parse::<u64>().unwrap());

        // query vault
        let msg = QueryMsg::GetVault {id: Uint64::new(1)};
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
