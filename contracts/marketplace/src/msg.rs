use crate::cw721::Cw721ReceiveMsg;
use cosmwasm_std::{Addr, Coin, Decimal, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub fee: Decimal,
    pub denom : String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Buy {
        offering_id: String,
        address:String
    },
    WithdrawNft {
        offering_id: String,
        address:String
    },
    ReceiveNft(Cw721ReceiveMsg),
    /// only admin.
    WithdrawFees {
        amount: Uint128,
        denom: String,
    },
    /// only admin.
    ChangeFee {
        fee: Decimal,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SellNft {
    pub list_price: Coin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetStateInfo{},
    GetCollectionInfo{address:String},
    GetFee {},
    /// With Enumerable extension.
    /// Requires pagination. Lists all offers controlled by the contract.
    /// Return type: OffersResponse.
    GetOffers {
        page_num: u32,
        count: u32,
        address:String
    },
    GetSaleHistory{
        page_num: u32,
        count: u32,
        address:String
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FeeResponse {
    pub fee: Decimal,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct OffersResponse {
    pub offers: Vec<Offer>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Offer {
    pub id: String,
    pub token_id: String,
    pub contract: Addr,
    pub seller: Addr,
    pub list_price: Coin,
}
