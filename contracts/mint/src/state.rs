use cosmwasm_std::{Uint128, Decimal,Coin};
use cw_storage_plus::{Map,Item};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONFIG : Item<State> = Item::new("config_state");
pub const USERINFO : Map<&str, Uint128> = Map::new("config_user_info");
pub const WHITEUSERS : Map<&str,Uint128>  = Map::new("config_white_user_info");
pub const CW721_ADDRESS : Item<String> = Item::new("config_collection_address");


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub admin:String,
    pub base_token_uri: String,
    pub total_supply: Uint128,
    pub mint_count:Uint128,
    pub start_mint_time: u64,
    pub per_address_limit: Uint128,
    pub unit_price: Coin,
    pub mint_flag:bool,
    pub enable_token_id:Option<Vec<u32>>,
    pub is_public_mint:bool,
    pub nft_base_name:String,
    pub is_rand_mint:bool
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AdminInfo {
    pub address:String,
    pub portion:Decimal
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Cw721InitMessage {
    pub name:String,
    pub symbol:String,
    pub minter:String
}

