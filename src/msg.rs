use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Uint128, Uint64};
use cw20::Cw20ReceiveMsg;
use crate::state::{Ledger};

#[cw_serde]
pub struct InstantiateMsg {
    /// cw20_addr is the address of the allowed cw20 token
    pub cw20_addr: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateVault {
        admin_addr: String,
    },
    /// Receive forwards received cw20 tokens to an execution logic
    Receive(Cw20ReceiveMsg),
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
    pub ledger_list: Vec<Ledger>
}
