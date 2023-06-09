use cosmwasm_std::{Uint128, Decimal,Coin};
use cw_storage_plus::{Map,Item};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONFIG : Item<State> = Item::new("config_state");
pub const USERINFO : Map<&str, Uint128> = Map::new("config_user_info");
pub const WHITEUSERS : Map<&str,Uint128>  = Map::new("config_white_user_info");
pub const CW721_ADDRESS : Item<String> = Item::new("config_collection_address");
pub const MIDDLEWARE: Item<String> = Item::new("config_middleware");


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub admin: String,
    pub base_token_uri: String,
    pub total_supply: Uint128,
    pub mint_count: Uint128,
    pub start_mint_time: u64,
    pub per_address_limit: Uint128,
    pub public_price: Coin,
    pub private_price: Coin,
    pub mint_flag: bool,
    pub enable_token_id: Option<Vec<u32>>,
    pub is_public_mint: bool,
    pub nft_base_name: String,
    pub base_image_uri: String,
    pub middleware_address: String,
    pub content_type: String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AdminInfo {
    pub address:String,
    pub portion:Decimal
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Cw721InitMessage {
    pub name: String,
    /// Symbol of the NFT contract
    pub symbol: String,
    /// The minter is the only one who can create new NFTs.
    pub minter:Option<String>,
    /// This is designed for a base NFT that is controlled by an external program
    /// or contract. You will likely replace this with custom logic in custom NFTs

    pub admin: String,

    pub collection_info : CollectionInfo,

    pub mint_info : Option<MintInfo>,

    pub royalty_info : Royalty
}


#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct CollectionInfo {
    pub title: String,
    pub background_url:String,
    pub logo_url:String,
    pub description:String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct MintInfo {
    pub base_token_uri: String,
    pub total_supply: Uint128,
    pub start_mint_time: u64,
    pub per_address_limit: Uint128,
    pub public_price: Coin,
    pub private_price:Coin,
    pub mint_flag:bool,
    pub is_public_mint:bool,
    pub nft_base_name:String,
    pub base_image_uri:String
}


#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct SocialLinkType {
    pub tool: String,
    pub link: String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Royalty {
    pub address: String,
    pub royalty_rate: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RegisterState {
    pub admin: String,
    pub nft_id: u64,
    pub collection_count: u64,
    pub minter_id: u64
}