use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128, Uint64};
use crate::state::{Ledger};
use cw20::Cw20ReceiveMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub cw20_addr: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateVault (),
    Withdraw {
        vaultId: Uint64,
        amount: Uint128,
    },
    // deposit. be executed when you send coin to this contract in CW20 contract
    Receive(Cw20ReceiveMsg),
}

#[cw_serde]
pub enum ReceiveMsg {
    Send { vault_id: Uint64 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(VaultResponse)]
    GetVault { id: Uint64 },
}

// We define a custom struct for each query response
#[cw_serde]
pub struct VaultResponse {
    pub admin_addr: String,
    pub collected: Uint128,
    pub ledger_list: Vec<Ledger>,
}
