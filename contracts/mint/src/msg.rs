use cosmwasm_std::{ Uint128,Coin};
use schemars::{JsonSchema};
use serde::{Deserialize, Serialize};

use crate::state::{Cw721InitMessage, State};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub admin:String,
    pub base_token_uri: String,
    pub total_supply: Uint128,
    pub mint_count:Uint128,
    pub cw721_code_id: u64,
    pub cw721_instantiate_msg: Cw721InitMessage,
    pub start_mint_time: u64,
    pub per_address_limit: Uint128,
    pub unit_price: Coin,
    pub enable_token_id:Option<Vec<u32>>,
    pub is_public_mint:bool,
    pub nft_base_name:String,
    pub white_user_list:Option<Vec<WhiteUserInfo>>,
    pub is_rand_mint:bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Mint{},
    SetConfig{config:State},
    SetMintFlag{flag:bool},
    AddWhiteUsers{white_users:Vec<WhiteUserInfo>}
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
