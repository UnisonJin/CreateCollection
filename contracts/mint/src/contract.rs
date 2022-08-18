use cosmwasm_std::{
    entry_point, to_binary,   CosmosMsg, Deps, DepsMut,Binary,SubMsg,
    Env, MessageInfo, Response, StdResult, Uint128, WasmMsg,BankMsg,Coin, ReplyOn,Reply
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg,Extension, InstantiateMsg, QueryMsg, WhiteUserInfo};
use crate::state::{
    CONFIG,State,  USERINFO, WHITEUSERS, CW721_ADDRESS
};
use crate::rand::{sha_256, Prng};

use rand::{RngCore, SeedableRng};
use rand_chacha::ChaChaRng;

use cw721_base::{ExecuteMsg as Cw721BaseExecuteMsg, MintMsg};
use cw2::{set_contract_version};
use cw_utils::{parse_reply_instantiate_data};


const CONTRACT_NAME: &str = "crates.io:sg-minter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const MAX_TOKEN_LIMIT : Uint128 = Uint128::new(10000);
const MAX_PER_ADDRESS_LIMIT : Uint128 = Uint128::new(100);
const INSTANTIATE_CW721_REPLY_ID : u64 = 1;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let creator = info.sender.to_string();
    let message = msg.cw721_instantiate_msg.mint_info.clone().unwrap();
    
    
    //check if the total_supply is more than zero and less than max token_limit
     if message.total_supply == Uint128::zero() || message.total_supply > MAX_TOKEN_LIMIT {
        return Err(ContractError::InvalidNumTokens {
            min: Uint128::new(1),
            max: MAX_TOKEN_LIMIT,
        });
    }

    //check the per address limit is more than zero and less than max per address limit
    if message.per_address_limit == Uint128::zero() || message.per_address_limit > MAX_PER_ADDRESS_LIMIT {
        return Err(ContractError::InvalidPerAddressLimit {
            max: MAX_PER_ADDRESS_LIMIT,
            min: Uint128::new(1),
            got: message.per_address_limit,
        });
    }
    
    // if current time is beyond the provided start time return error
    let current_time = env.block.time.seconds();
    if current_time > message.start_mint_time {
        return Err(ContractError::InvalidStartTime(
            message.start_mint_time,
            current_time,
        ));
    }

    let total_num = Uint128::u128(&message.total_supply) as u32;
    let mut mintable_token_list:Vec<u32> = Vec::new();      
    for i in 1 .. total_num+1{
        mintable_token_list.push(i);
    }
    
    let state = State {
        admin:creator.clone(),
        base_token_uri: message.base_token_uri,
        total_supply: message.total_supply,
        mint_count:Uint128::zero(),
        start_mint_time: message.start_mint_time,
        per_address_limit: message.per_address_limit,
        public_price: message.public_price,
        private_price:message.private_price,
        mint_flag:true,
        enable_token_id:Some(mintable_token_list),
        is_public_mint:message.is_public_mint,
        nft_base_name:message.nft_base_name.clone(),
        base_image_uri:message.base_image_uri
    };
    CONFIG.save(deps.storage, &state)?;

    // message to instantiate the new nft collection contract
    let init_msg : SubMsg = SubMsg{
            msg: WasmMsg::Instantiate { 
            admin: Some(creator), 
            code_id: msg.cw721_code_id, 
            msg: to_binary(&msg.cw721_instantiate_msg)?,
            funds: info.funds, 
            label: message.nft_base_name 
        }.into(),
        id:INSTANTIATE_CW721_REPLY_ID,
        gas_limit:None,
        reply_on:ReplyOn::Success
    } ;

   
    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract_name", CONTRACT_NAME)
        .add_attribute("contract_version", CONTRACT_VERSION)
        .add_attribute("sender", info.sender)
        .add_submessage(init_msg)
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
        ExecuteMsg::Mint{} => execute_mint(deps, env, info),
        ExecuteMsg::SetConfig { config }   => execute_set_config(deps, info,config),
        ExecuteMsg::SetMintFlag {  flag } => execute_set_flag(deps, info,flag),
        ExecuteMsg::AddWhiteUsers {  white_users } => execute_add_white_users(deps, info, white_users),
        ExecuteMsg::ChangeSaleType { is_public_mint }  => execute_change_sale_type(deps, info, is_public_mint)
    }                                  
}

fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let sender = info.sender.to_string();

    let collection_info = CONFIG.load(deps.storage)?;

    //check if mint is enabled
    if collection_info.mint_flag == false{
        return Err(ContractError::MintDisabled {  });
    }

    //check if the current time is passed the mint start time
    if collection_info.start_mint_time>env.block.time.seconds(){
        return Err(ContractError::MintNotStarted {});
    }

    //check if all nfts are minted 
    if collection_info.mint_count >=  collection_info.total_supply {
        return Err(ContractError::SoldOut {});
    }

    let user_mint_count =  WHITEUSERS.may_load(deps.storage, &sender)?;

    //check the white_list_user if this is not public mint
    if !collection_info.is_public_mint && user_mint_count == None{
        return Err(ContractError::NotWhiteListedUser{})
    }

    //check the per_wallet limit minting number and reset the current mint count of the user
    if collection_info.is_public_mint{
        let user_mint_count = USERINFO.may_load(deps.storage, &sender)?;
        if user_mint_count == None{
            USERINFO.save(deps.storage, &sender, &Uint128::new(1))?;
        } else{
            if user_mint_count.unwrap() >= collection_info.per_address_limit{
                return Err(ContractError::MaxPerAddressLimitExceeded{});
            } else {
                USERINFO.update(deps.storage, &sender, 
                    |user_mint_count|->StdResult<_>{
                        let mut user_mint_count = user_mint_count.unwrap();
                        user_mint_count = user_mint_count + Uint128::new(1);
                        Ok(user_mint_count)
                    })?;
            }
        }
    } else{
        let user_mint_count =  WHITEUSERS.load(deps.storage, &sender.clone())?;
        if user_mint_count == Uint128::zero(){
            return Err(ContractError::MaxPerAddressLimitExceeded {  });
        } else {
            WHITEUSERS.update(deps.storage, &sender.clone(), 
            |white_user_info| -> StdResult<_>{
                let mut white_user_info =  white_user_info.unwrap();
                white_user_info = white_user_info - Uint128::new(1);
                Ok(white_user_info)
            })?;
        }
    }

    //get mint index for token_id and change collection info
    let  mint_index:Uint128;

    let mut enable_token_id = collection_info.enable_token_id.unwrap();
    let count = enable_token_id.len();
    let prng_seed: Vec<u8> = sha_256(base64::encode("entropy").as_bytes()).to_vec();
    let random_seed = new_entropy(&info,&env, prng_seed.as_ref(), prng_seed.as_ref());
    let mut rng = ChaChaRng::from_seed(random_seed);
    let  rand_num = (rng.next_u32() % (count as u32)) as usize ;
    let rand = enable_token_id[rand_num];
    mint_index = Uint128::new(rand as u128);
    enable_token_id.remove(rand_num);

    // updated mintable token ids
    CONFIG.update(deps.storage, |mut collection_info|->StdResult<_>{  
            collection_info.enable_token_id = Some(enable_token_id);
            Ok(collection_info)
        })?;


    //increase the total mint count by one
    CONFIG.update(deps.storage, |mut collection_info|->StdResult<_>{  
            collection_info.mint_count = collection_info.mint_count + Uint128::new(1);
            Ok(collection_info)
    })?;

    let token_id = [collection_info.nft_base_name,mint_index.to_string()].join(".");
    let collection_address = CW721_ADDRESS.load(deps.storage)?;

    //mint message
    let mut messages :Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: collection_address,
            msg: to_binary(&Cw721BaseExecuteMsg::Mint(MintMsg {
                //::<Metadata>
                token_id: token_id.clone(),
                owner: sender.clone(),
                token_uri: Some([[collection_info.base_token_uri,mint_index.to_string()].join(""),"json".to_string()].join(".")),
                extension:  Extension{
                    minter:Some(sender.clone()),
                    image_url:Some([[collection_info.base_image_uri,mint_index.to_string()].join(""),"png".to_string()].join("."))
                }
            }))?,
            funds: vec![],
    }));

    let denom :String;
    let unit_price:Uint128;
    if collection_info.is_public_mint{
        denom = collection_info.public_price.denom;
        unit_price = collection_info.public_price.amount;
    }else{
        denom = collection_info.private_price.denom;
        unit_price = collection_info.private_price.amount;
    }
    //funds the users sent
    let amount=  info
        .funds
        .iter()
        .find(|c| c.denom == denom)
        .map(|c| Uint128::from(c.amount))
        .unwrap_or_else(Uint128::zero);
    
    //check the price(for the owner the price is zero) 
    if sender == collection_info.admin{
        if amount > Uint128::zero(){
            return Err(ContractError::NotExactFunds{
                price:Uint128::zero(),
                sent_money: amount
            })   
        }
    } else{
        if amount != unit_price{
            return Err(ContractError::NotExactFunds{
                price:unit_price,
                sent_money:amount
            })
        }else{ messages.push(CosmosMsg::Bank(BankMsg::Send { 
                to_address: collection_info.admin,
                amount: vec![Coin{
                    denom:denom,
                    amount:amount
                }] })
            )}
    }
   
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("action", "mint")
        .add_attribute("minter", "sender")
        .add_attribute("token_id", token_id)
    )
}

fn execute_set_config(
    deps: DepsMut,
    info: MessageInfo,
    config:State
)->Result<Response,ContractError>{

    let state = CONFIG.load(deps.storage)?;

    //auth check
    if info.sender.to_string() != state.admin{
        return Err(ContractError::Unauthorized {});
    }

    CONFIG.update(deps.storage,
    |mut state|->StdResult<_>{
        state.base_token_uri = config.base_token_uri;
        state.start_mint_time = config.start_mint_time;
        state.per_address_limit = config.per_address_limit;
        state.public_price = config.public_price;
        state.private_price = config.private_price;
        state.nft_base_name = config.nft_base_name;
        Ok(state)
    })?;

    Ok(Response::new()
        .add_attribute("action", "change the config"))
}



fn execute_set_flag(
    deps: DepsMut,
    info: MessageInfo,
    flag:bool
) -> Result<Response, ContractError> {
   let state =CONFIG.load(deps.storage)?;
   //auth check
   if state.admin != info.sender.to_string() {
        return Err(ContractError::Unauthorized {});
   }
   
   CONFIG.update(deps.storage, 
    |mut state|->StdResult<_>{
        state.mint_flag = flag;
        Ok(state)
    })?;

   Ok(Response::new()
       .add_attribute("action", "set mint flag"))
}


fn execute_change_sale_type(
    deps: DepsMut,
    info: MessageInfo,
    is_public_mint:bool
) -> Result<Response, ContractError> {
   let state =CONFIG.load(deps.storage)?;
   //auth check
   if state.admin != info.sender.to_string() {
        return Err(ContractError::Unauthorized {});
   }
   
   CONFIG.update(deps.storage, 
    |mut state|->StdResult<_>{
        state.is_public_mint = is_public_mint;
        Ok(state)
    })?;

   Ok(Response::new()
       .add_attribute("action", "swtich mint type"))
}




fn execute_add_white_users(
    deps: DepsMut,
    info: MessageInfo,
    white_users:Vec<WhiteUserInfo>
) -> Result<Response, ContractError> {
   let state =CONFIG.load(deps.storage)?;
   
   //auth check
    if state.admin != info.sender.to_string() {
        return Err(ContractError::Unauthorized {});
    }
   
    for white_user in white_users{
        deps.api.addr_validate(&white_user.address)?;
        WHITEUSERS.save(deps.storage, &white_user.address, &white_user.wallet_limit)?;
    }


    Ok(Response::new()
        .add_attribute("action", "add white users")
    )
}




// Reply callback triggered from cw721 contract instantiation
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    if msg.id != INSTANTIATE_CW721_REPLY_ID {
        return Err(ContractError::InvalidReplyID {});
    }

    let reply = parse_reply_instantiate_data(msg);
    match reply {
        Ok(res) => {
            CW721_ADDRESS.save(deps.storage, &res.contract_address)?;
            Ok(Response::default().add_attribute("action", "instantiate_cw721_reply")
                .add_attribute("nft_address", res.contract_address))
        }
        Err(_) => Err(ContractError::InstantiateCw721Error {}),
    }
}


#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetStateInfo {} => to_binary(& query_state_info(deps)?),
        QueryMsg::GetUserInfo { address }=>to_binary(& query_user_info(deps,address)?),
    }
}


pub fn query_state_info(deps:Deps) -> StdResult<State>{
    let state = CONFIG.load(deps.storage)?;
    Ok(state)
}

pub fn query_user_info(deps:Deps, address:String) -> StdResult<Uint128>{
   let state = CONFIG.load(deps.storage)?;
   if state.is_public_mint{
     let user_info = USERINFO.may_load(deps.storage, &address)?;    
     if user_info == None{
        Ok(Uint128::zero()) 
     } else{
        Ok(user_info.unwrap())
     }
   } else{
     let user_info = WHITEUSERS.may_load(deps.storage, &address)?; 
     if user_info == None{
        Ok(Uint128::zero())
     }
     else{
        Ok(user_info.unwrap())
     }
   }
}



pub fn new_entropy(info:&MessageInfo,env: &Env, seed: &[u8], entropy: &[u8]) -> [u8; 32] {
    // 16 here represents the lengths in bytes of the block height and time.
    let entropy_len = 16 + info.sender.to_string().len() + entropy.len();
    let mut rng_entropy = Vec::with_capacity(entropy_len);
    rng_entropy.extend_from_slice(&env.block.height.to_be_bytes());
    rng_entropy.extend_from_slice(&info.sender.as_bytes());
    rng_entropy.extend_from_slice(entropy);

    let mut rng = Prng::new(seed, &rng_entropy);

    rng.rand_bytes()
}

