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

pub const VAULTS: Map<Addr, Vault> = Map::new("vault");