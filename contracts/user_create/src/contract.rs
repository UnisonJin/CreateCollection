use cosmwasm_std::{
    entry_point, to_binary,    Deps, DepsMut,Binary,SubMsg,QueryRequest,WasmQuery,
    Env, MessageInfo, Response, StdResult, WasmMsg, ReplyOn,Reply
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, Cw721BaseQueryMsg, AdminResponse, MinterQueryMsg};
use crate::state::{
    CONFIG,State,  CollectionInfoMessage, Cw721InitMessage, CONTRACTLIST, CollectionDetailInfo, MintInitMsg
};

use cw2::{set_contract_version};
use cw_utils::{parse_reply_instantiate_data};


const CONTRACT_NAME: &str = "crates.io:create_collection";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const INSTANTIATE_CW721_REPLY_ID : u64 = 1;
const INSTANTIATE_MINTER_ID : u64 = 2;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let admin = info.sender.to_string();
    let state = State{
        nft_id:msg.nft_id,
        minter_id:msg.minter_id,
        admin:admin,
        collection_count:0
    };
    CONFIG.save(deps.storage, &state)?;
  
    Ok(Response::new()
        .add_attribute("action", "instantiate")
    )
       
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::AddUserCollection { collection_info }  => execute_add_user_collection(deps,env,info,collection_info),
        ExecuteMsg::AddAdminCollection { collection_info, content_type } => execute_add_admin_collection(deps, env, info, collection_info, content_type),
        ExecuteMsg::SetNftId { id } => execute_set_id(deps,env,info,id),
        ExecuteMsg::ChangeAdmin { address } => execute_change_admin(deps,env,info,address),
        ExecuteMsg::SetMinterId { id }  => execute_set_minter_id(deps,env,info,id)
    }                                  
}


fn execute_add_user_collection(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection_info: CollectionInfoMessage
)->Result<Response,ContractError>{

    let state = CONFIG.load(deps.storage)?;
    
    let sender =  info.sender.to_string();

    let cw721_init_msg = Cw721InitMessage{
        name:collection_info.name.clone(),
        symbol:collection_info.symbol,
        minter:Some(sender.clone()),
        admin:sender.clone(),
        collection_info:collection_info.collection_info,
        mint_info:None,
        royalty_info:collection_info.royalty_info
    };

      // message to instantiate the new nft collection contract
    let init_msg : SubMsg = SubMsg{
            msg: WasmMsg::Instantiate { 
            admin: Some(sender.clone()), 
            code_id: state.nft_id, 
            msg: to_binary(&cw721_init_msg)?,
            funds: vec![], 
            label: collection_info.name 
        }.into(),
        id:INSTANTIATE_CW721_REPLY_ID,
        gas_limit:None,
        reply_on:ReplyOn::Success
    } ;

    Ok(Response::new()
        .add_attribute("action", "add_user_collection")
        .add_submessage(init_msg))
}



fn execute_add_admin_collection(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collection_info: CollectionInfoMessage,
    content_type: String
)->Result<Response,ContractError>{
    let state = CONFIG.load(deps.storage)?;

    let sender =  info.sender.to_string();

    //auth check
    if info.sender.to_string() != state.admin{
        return Err(ContractError::Unauthorized {});
    }  

    let cw721_init_msg = Cw721InitMessage{
        name:collection_info.name.clone(),
        symbol:collection_info.symbol,
        minter:None,
        admin:sender.clone(),
        collection_info:collection_info.collection_info,
        mint_info:collection_info.mint_info,
        royalty_info:collection_info.royalty_info
    };

    let mint_msg = MintInitMsg{
        cw721_instantiate_msg:cw721_init_msg,
        cw721_code_id:state.nft_id,
        content_type
    };

    // message to instantiate the new nft collection contract
    let init_msg : SubMsg = SubMsg{
            msg: WasmMsg::Instantiate { 
            admin: Some(sender.clone()), 
            code_id: state.minter_id, 
            msg: to_binary(&mint_msg)?,
            funds: vec![], 
            label: collection_info.name 
        }.into(),
        id:INSTANTIATE_MINTER_ID,
        gas_limit:None,
        reply_on:ReplyOn::Success
    } ;

    Ok(Response::new()
        .add_attribute("action", "add_admin_collection")
        .add_submessage(init_msg)
     )
}




fn execute_set_id(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: u64
)->Result<Response,ContractError>{

    let state = CONFIG.load(deps.storage)?;

    //auth check
    if info.sender.to_string() != state.admin{
        return Err(ContractError::Unauthorized {});
    }

    CONFIG.update(deps.storage, 
        |mut state| -> StdResult<_> {
            state.nft_id = id;
            Ok(state)
    })?;

    Ok(Response::new()
        .add_attribute("action", "set nft id"))
}


fn execute_set_minter_id(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: u64
)->Result<Response,ContractError>{

    let state = CONFIG.load(deps.storage)?;

    //auth check
    if info.sender.to_string() != state.admin{
        return Err(ContractError::Unauthorized {});
    }

    CONFIG.update(deps.storage, 
        |mut state| -> StdResult<_> {
            state.minter_id = id;
            Ok(state)
    })?;

    Ok(Response::new()
        .add_attribute("action", "set_nft_id"))
}



fn execute_change_admin(
    deps: DepsMut,
    _env:Env,
    info: MessageInfo,
    address:String
) -> Result<Response, ContractError> {
   let state =CONFIG.load(deps.storage)?;
   //auth check
   if state.admin != info.sender.to_string() {
        return Err(ContractError::Unauthorized {});
   }
   
   CONFIG.update(deps.storage, 
    |mut state|->StdResult<_>{
        state.admin = address;
        Ok(state)
    })?;

   Ok(Response::new()
       .add_attribute("action", "change_admin"))
}



// Reply callback triggered from cw721 contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.id  {
        INSTANTIATE_CW721_REPLY_ID =>  
            {
                let reply = parse_reply_instantiate_data(msg);
                match reply {
                    Ok(res) => {
                        let state = CONFIG.load(deps.storage)?;

                        let admin_info:AdminResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                            contract_addr: res.contract_address.clone(),
                            msg: to_binary(&Cw721BaseQueryMsg::Admin {  })?,
                        }))?;

                        let admin = admin_info.admin;
                    
                        CONTRACTLIST.save(deps.storage, &( state.collection_count + 1 ).to_string(), &CollectionDetailInfo{
                            address:res.contract_address.clone(),
                            is_rand:false,
                            creator:admin
                        })?;
                        
                        CONFIG.update(deps.storage, |mut state|-> StdResult<_>{
                            state.collection_count = state.collection_count + 1;
                            Ok(state)
                        })?;
                        
                        Ok(Response::default().add_attribute("action", "add_user_collection")
                            .add_attribute("nft_address", res.contract_address))
                    }
                    Err(_) => Err(ContractError::InstantiateCw721Error {}),
                }
        },
        INSTANTIATE_MINTER_ID => {
             let reply = parse_reply_instantiate_data(msg);
                match reply {
                    Ok(res) => {
                        let state = CONFIG.load(deps.storage)?;       
                        let address = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                                contract_addr: res.contract_address,
                                msg: to_binary(&MinterQueryMsg::GetCollectionAddress {  })?,
                        }))?;
                       
                        CONTRACTLIST.save(deps.storage, &(state.collection_count + 1 ).to_string(), &CollectionDetailInfo{
                                    address:address,
                                    is_rand:true,
                                    creator:state.admin
                        })?;
                                
                        CONFIG.update(deps.storage, |mut state|-> StdResult<_>{
                            state.collection_count = state.collection_count + 1;
                            Ok(state)
                        })?;

                        Ok(Response::new()
                            .add_attribute("action", "add_admin_collection")
                        )
                    }
                    Err(_) => Err(ContractError::InstantiateCw721Error {}),
                }
        },
        _id => Err(ContractError::InvalidReplyID {  }),

    }
}


#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetStateInfo {} => to_binary(& query_state_info(deps)?),
        QueryMsg::GetCollections { id }  => to_binary(& query_collections(deps,id)?)
    }
}


pub fn query_state_info(deps:Deps) -> StdResult<State>{
    let state = CONFIG.load(deps.storage)?;
    Ok(state)
}



pub fn query_collections(deps:Deps, id:Vec<String>) -> StdResult<Vec<CollectionDetailInfo>>{
   let mut result : Vec<CollectionDetailInfo> = Vec::new();
   for count in id{
      let collection_info =  CONTRACTLIST.may_load(deps.storage, &count)?;
      if collection_info != None{
        result.push(collection_info.unwrap());
      }
   }
   Ok(result)
}

