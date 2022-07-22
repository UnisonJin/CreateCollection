use cosmwasm_std::Uint128;
use cw_storage_plus::{Map,Item};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


pub const CONFIG: Item<State> = Item::new("config_state");
pub const ADMININFO: Map<&str, Vec<AdminInfo>> = Map::new("admins");
pub const USERINFO: Map<(&str,&str), Uint128> = Map::new("user_info");
pub const COLLECTIONINFO : Map<&str, CollectionInfo> = Map::new("collection_info");


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub owner:String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AdminInfo {
    pub address:String,
    pub amount:Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionInfo {
    pub total_nft:Uint128,
    pub mint_count : Uint128,
    pub check_mint:Vec<u32>,
    pub url :String,
    pub image_url:String,
    pub price:Uint128,
    pub denom:String,
    pub max_nft:Uint128,
    pub name:String,
    pub can_mint:bool,
    pub is_rand_mint:bool
}

