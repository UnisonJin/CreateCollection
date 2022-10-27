use schemars::{JsonSchema};
use serde::{Deserialize, Serialize};

use crate::state::{ CollectionInfoMessage};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
  pub nft_id:u64,
  pub minter_id:u64
} 

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AddUserCollection{ collection_info : CollectionInfoMessage },
    AddAdminCollection{ collection_info : CollectionInfoMessage, content_type: String },
    SetNftId{ id:u64 },
    ChangeAdmin{ address:String },
    SetMinterId{ id:u64 }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
      GetStateInfo{},
      GetCollections{id:Vec<String>}
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MinterQueryMsg{
      GetCollectionAddress{}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw721BaseQueryMsg {
      Admin{},
}


#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct AdminResponse {
    pub admin: String,
}