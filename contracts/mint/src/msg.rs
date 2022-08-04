use cosmwasm_std::{ Uint128,Coin};
use schemars::{JsonSchema};
use serde::{Deserialize, Serialize};

use crate::state::{Cw721InitMessage, State};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub cw721_instantiate_msg: Cw721InitMessage,
    pub cw721_code_id:u64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Mint{},
    SetConfig{config:State},
    SetMintFlag{flag:bool},
    AddWhiteUsers{white_users:Vec<WhiteUserInfo>},
    ChangeSaleType{is_public_mint:bool}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
      GetStateInfo{},
      GetUserInfo{address:String},
    }

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Extension {   
    pub minter: Option<String>,
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct WhiteUserInfo {   
    pub address: String,
    pub wallet_limit:Uint128
}
