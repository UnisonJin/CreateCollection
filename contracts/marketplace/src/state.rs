use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::ContractError;
use cosmwasm_std::{Addr, Api, Coin, Decimal, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub fee: Decimal,
    pub owner: Addr,
    pub tvl:Uint128,
    pub denom:String
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Offering {
    pub token_id: String,
    pub contract: Addr,
    pub seller: Addr,
    pub list_price: Coin,
}

pub const STATE: Item<State> = Item::new("state");
pub const OFFERINGS: Map<(&str,&str), Offering> = Map::new("offerings");
pub const SALEHISTORY : Map<(&str,&str),SaleHistoryInfo> = Map::new("sale history");
pub const COLLECTIONINFO : Map<&str,CollectionInfo> = Map::new("collection_info");


pub fn increment_offerings(store: &mut dyn Storage,address:String) -> Result<u64, ContractError> {
    let mut num = 0;
    COLLECTIONINFO.update(store,&address ,| collection_info| -> Result<_, ContractError> {
       if collection_info!= None{
            let mut collection_info = collection_info.unwrap();
            collection_info.num_offerings += 1;
            num = collection_info.num_offerings;
            Ok(collection_info)
        }
       else{
        num = 1;
        Ok(CollectionInfo {
            sale_id:0 , 
            tvl: Uint128::new(0),
            num_offerings: 1 
          })
       }
    })?;

    Ok(num)
}

pub fn get_fund(funds: Vec<Coin>, denom: String) -> Result<Coin, ContractError> {
    for fund in funds.into_iter() {
        if fund.denom == denom {
            return Ok(fund);
        }
    }

    Err(ContractError::InsufficientFunds {})
}

pub fn maybe_addr(api: &dyn Api, human: Option<String>) -> StdResult<Option<Addr>> {
    human.map(|x| api.addr_validate(&x)).transpose()
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SaleHistoryInfo {
    pub from :String,
    pub to: String,
    pub denom:String,
    pub amount:Uint128,
    pub time : u64,
    pub nft_address:String,
    pub token_id:String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CollectionInfo{
    pub sale_id : u64,
    pub tvl: Uint128,
    pub num_offerings: u64,
}
