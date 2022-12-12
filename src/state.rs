use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, DepsMut, StdResult, Uint128, Uint64, Timestamp};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub owner: Addr,
    pub cw20_addr: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct Vault {
    pub admin_addr: Addr,
    pub collected: Uint128,
    pub ledger_list: Vec<Ledger>
}

#[cw_serde]
pub struct Ledger {
    pub coin_amount: Uint128,
    pub receive_time: Timestamp,
}

/// VAULT_SEQ holds the last vault ID
pub const VAULT_SEQ: Item<u64> = Item::new("vault_seq");
pub const VAULTS: Map<u64, Vault> = Map::new("vault");

pub fn save_vault(deps: DepsMut, vault: &Vault) -> StdResult<u64> {
    // increment id if exists, or return 1
    let id = VAULT_SEQ.load(deps.storage)?;
    let id = Uint64::new(id).checked_add(Uint64::new(1))?.u64();
    VAULT_SEQ.save(deps.storage, &id)?;
    VAULTS.save(deps.storage, id, vault)?;
    Ok(id)
}