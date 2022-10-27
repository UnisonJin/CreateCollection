use cosmwasm_std::{Uint128, Decimal,Coin};
use cw_storage_plus::{Map,Item};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const CONFIG : Item<State> = Item::new("config_state");
pub const CONTRACTLIST : Map<&str, CollectionDetailInfo> = Map::new("config_user_info");


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub admin: String,
    pub nft_id: u64,
    pub collection_count: u64,
    pub minter_id: u64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AdminInfo {
    pub address:String,
    pub portion:Decimal
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Cw721InitMessage {
     /// Name of the NFT contract
    pub name: String,
    /// Symbol of the NFT contract
    pub symbol: String,
    /// The minter is the only one who can create new NFTs.
    pub minter:Option<String> ,
    /// This is designed for a base NFT that is controlled by an external program
    /// or contract. You will likely replace this with custom logic in custom NFTs

    pub admin: String,

    pub collection_info : CollectionInfo,

    pub mint_info : Option<MintInfo>,

    pub royalty_info : Royalty

}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionInfoMessage {
    pub name: String,
    pub symbol: String,
    pub collection_info: CollectionInfo,
    pub royalty_info: Royalty,
    pub mint_info : Option<MintInfo>
}


#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct CollectionInfo {
    pub title: String,
    pub background_url: String,
    pub logo_url: String,
    pub description: String
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



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Royalty {
    pub address: String,
    pub royalty_rate: Decimal,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionDetailInfo {
    pub address: String,
    pub is_rand: bool,
    pub creator: String
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintInitMsg {
     pub cw721_instantiate_msg: Cw721InitMessage,
     pub cw721_code_id: u64,
     pub content_type: String
}
