use cosmwasm_std::{
    entry_point, to_binary,   CosmosMsg, Deps, DepsMut,Binary,
    Env, MessageInfo, Response, StdResult, Uint128, WasmMsg,BankMsg,Coin
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg,Image, InstantiateMsg, QueryMsg};
use crate::state::{
    CONFIG,ADMININFO,State, AdminInfo, USERINFO,COLLECTIONINFO, CollectionInfo, self
};
use crate::rand::{sha_256, Prng};

use cw721_base::{ExecuteMsg as Cw721BaseExecuteMsg, MintMsg};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaChaRng;


#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner:msg.owner,
    };
    CONFIG.save(deps.storage, &state)?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Mint{address} => execute_mint(deps, env, info,address),
        ExecuteMsg::ChangeOwner { address } => execute_chage_owner(deps, info, address),
        ExecuteMsg::AddCollection { members, nft_address,collection}  => execute_add_collection(deps, info,members, nft_address,collection),
        ExecuteMsg::UpdateCollection { members, nft_address,collection}  => execute_update_collection(deps, info,members, nft_address,collection),
        ExecuteMsg::SetMintFlag { address, flag } => execute_set_flag(deps, info, address,flag)
    }
}

fn execute_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address:String
) -> Result<Response, ContractError> {
    //address check
    deps.api.addr_validate(&address)?;
    let sender = info.sender.to_string();

    //Check if this nft collection is available
    let collection_info = COLLECTIONINFO.may_load(deps.storage, &address)?;

    if collection_info == None{
        return Err(ContractError::CollectionNotFound { });
    }

    let  collection_info = collection_info.unwrap();

    if !collection_info.can_mint{
        return Err(ContractError::MintNotStarted {  });
    }

    if collection_info.mint_count >= collection_info.total_nft {
        return Err(ContractError::MintEnded {});
    }

    //check maximum nft per wallet.

   
    let count = USERINFO.may_load(deps.storage,(&address,&sender))?;
  
    if count == None{
        USERINFO.save(deps.storage,(&address,&sender), &Uint128::new(1))?;
    }

    else {
        let count = count.unwrap() + Uint128::new(1);
        USERINFO.save(deps.storage,(&address,&sender), &count)?;
        if count > collection_info.max_nft{
            return Err(ContractError::MintExceeded {  })
        }
    }

    //funds check

    let amount= info
        .funds
        .iter()
        .find(|c| c.denom == collection_info.denom)
        .map(|c| Uint128::from(c.amount))
        .unwrap_or_else(Uint128::zero);

    if amount != collection_info.price{
        return Err(ContractError::Notenough {});
    }

    let mut check_mint = collection_info.check_mint;
    let mut mint_index = Uint128::new(1) + collection_info.mint_count;

    if collection_info.is_rand_mint
     {
        let count = check_mint.len();
        let prng_seed: Vec<u8> = sha_256(base64::encode("entropy").as_bytes()).to_vec();
        let random_seed = new_entropy(&info,&env, prng_seed.as_ref(), prng_seed.as_ref());
        let mut rng = ChaChaRng::from_seed(random_seed);
        let  rand_num = (rng.next_u32() % (count as u32)) as usize ;
        let rand = check_mint[rand_num];
        mint_index = Uint128::new(rand as u128);
        check_mint.remove(rand_num);
    }
    
 
    let token_id = [collection_info.name,mint_index.to_string()].join(".");
  
    COLLECTIONINFO.update(deps.storage, &address,|collection_info|->StdResult<_>{
        let mut collection_info = collection_info.unwrap();    
        collection_info.mint_count = collection_info.mint_count+Uint128::new(1);
        collection_info.check_mint = check_mint;
        Ok(collection_info)
    })?;

    let admins = ADMININFO.load(deps.storage,&address)?;
    let mut messages:Vec<CosmosMsg> = vec![];

    for admin in admins {
        messages.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: admin.address,
                amount:vec![Coin{
                    denom:collection_info.denom.clone(),
                    amount:admin.amount
                }]
        }));
    }
   

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: address,
            msg: to_binary(&Cw721BaseExecuteMsg::Mint(MintMsg {
                //::<Metadata>
                token_id: token_id.clone(),
                owner: sender,
                token_uri: Some([[collection_info.url,mint_index.to_string()].join(""),"json".to_string()].join(".")),
                extension:  Image{
                    image:Some([[collection_info.image_url,mint_index.to_string()].join(""),"png".to_string()].join("."))
                }
            }))?,
            funds: vec![],
        }))
        .add_messages(messages)
    )
    
}


fn execute_chage_owner(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
   let state =CONFIG.load(deps.storage)?;
    if state.owner != info.sender.to_string() {
        return Err(ContractError::Unauthorized {});
    }
    CONFIG.update(deps.storage,
        |mut state|-> StdResult<_>{
            state.owner = address;
            Ok(state)
        }
    )?;
    Ok(Response::default())
}

fn execute_set_flag(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
    flag:bool
) -> Result<Response, ContractError> {
   let state =CONFIG.load(deps.storage)?;
    if state.owner != info.sender.to_string() {
        return Err(ContractError::Unauthorized {});
    }
    let collection_info = COLLECTIONINFO.may_load(deps.storage, &address)?;
    if collection_info == None{
        return Err(ContractError::CollectionNotFound {  })
    }
    COLLECTIONINFO.update(deps.storage, &address, 
        |collection_info|->StdResult<_>{
            let mut collection_info = collection_info.unwrap();
            collection_info.can_mint =flag;
            Ok(collection_info)
        })?;
    Ok(Response::default())
}



fn execute_add_collection(
    deps: DepsMut,
    info: MessageInfo,
    members: Vec<AdminInfo>,
    nft_address:String,
    collection:CollectionInfo
)->Result<Response,ContractError>{

    let state = CONFIG.load(deps.storage)?;

    deps.api.addr_validate(&nft_address)?;

    if info.sender.to_string() != state.owner{
        return Err(ContractError::Unauthorized {});
    }

    let total_count =Uint128::u128(&collection.total_nft) as u32;
    let check_length =  collection.check_mint.len() as u32;

    if total_count != check_length{
        return Err(ContractError::WrongNumber {  })
    }

    
    let mut total = Uint128::new(0);
    for admin in members.clone(){
        deps.api.addr_validate(&admin.address)?;
        total = total+admin.amount;
    }

    if total!= collection.price{
        return Err(ContractError::WrongPortion {  })
    }

    ADMININFO.save(deps.storage,&nft_address,&members)?;

    COLLECTIONINFO.save(deps.storage,&nft_address,&CollectionInfo{
       total_nft:collection.total_nft,
       mint_count:Uint128::new(0),
       url:collection.url,
       check_mint:collection.check_mint,
       image_url:collection.image_url,
       price:collection.price,
       denom:collection.denom,
       max_nft:collection.max_nft,
       name:collection.name,
       can_mint:true,
       is_rand_mint:collection.is_rand_mint
    })?;
    Ok(Response::default())
}

fn execute_update_collection(
    deps: DepsMut,
    info: MessageInfo,
    members: Vec<AdminInfo>,
    nft_address:String,
    collection:CollectionInfo
)->Result<Response,ContractError>{

    let state = CONFIG.load(deps.storage)?;

    deps.api.addr_validate(&nft_address)?;

    if info.sender.to_string() != state.owner{
        return Err(ContractError::Unauthorized {});
    }

    let collection_info = COLLECTIONINFO.may_load(deps.storage, &nft_address)?;
    if collection_info == None{
        return Err(ContractError::CollectionNotFound {  })
    }
    let collection_info = collection_info.unwrap();
    
    let mut total = Uint128::new(0);
    for admin in members.clone(){
        deps.api.addr_validate(&admin.address)?;
        total = total+admin.amount;
    }

    if total!= collection.price{
        return Err(ContractError::WrongPortion {  })
    }

    ADMININFO.save(deps.storage,&nft_address,&members)?;

    COLLECTIONINFO.save(deps.storage,&nft_address,&CollectionInfo{
       total_nft:collection.total_nft,
       mint_count:collection_info.mint_count,
       url:collection.url,
       check_mint:collection_info.check_mint,
       image_url:collection.image_url,
       price:collection.price,
       denom:collection.denom,
       max_nft:collection.max_nft,
       name:collection.name,
       is_rand_mint:collection.is_rand_mint,
       can_mint:true
    })?;
    Ok(Response::default())
}



fn new_entropy(info:&MessageInfo,env: &Env, seed: &[u8], entropy: &[u8]) -> [u8; 32] {
    // 16 here represents the lengths in bytes of the block height and time.
    let entropy_len = 16 + info.sender.to_string().len() + entropy.len();
    let mut rng_entropy = Vec::with_capacity(entropy_len);
    rng_entropy.extend_from_slice(&env.block.height.to_be_bytes());
    rng_entropy.extend_from_slice(&info.sender.as_bytes());
    rng_entropy.extend_from_slice(entropy);

    let mut rng = Prng::new(seed, &rng_entropy);

    rng.rand_bytes()
}




#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetStateInfo {} => to_binary(& query_state_info(deps)?),
        QueryMsg::GetAdminInfo { nft_address }=>to_binary(& query_admin_info(deps,nft_address)?),
        QueryMsg::GetUserInfo {nft_address, address }=>to_binary(& query_user_info(deps,nft_address,address)?),
        QueryMsg::GetCollectionInfo { nft_address }=>to_binary(& query_collection_info(deps,nft_address)?)
    }
}


pub fn query_state_info(deps:Deps) -> StdResult<State>{
    let state = CONFIG.load(deps.storage)?;
    Ok(state)
}

pub fn query_admin_info(deps:Deps,nft_address:String) -> StdResult<Vec<AdminInfo>>{
   let admin = ADMININFO.load(deps.storage,&nft_address)?;
   Ok(admin)
}

pub fn query_user_info(deps:Deps, nft_address:String,address:String) -> StdResult<Uint128>{
   let user_info = USERINFO.may_load(deps.storage, (&nft_address,&address))?;
   if user_info == None{
    Ok(Uint128::new(0))
   }
   else{
   Ok(user_info.unwrap())
    }
}

pub fn query_collection_info(deps:Deps,nft_address:String) -> StdResult<CollectionInfo>{
   let collection_info = COLLECTIONINFO.load(deps.storage,&nft_address)?;
   Ok(collection_info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{CosmosMsg, from_binary};

    #[test]
    fn buy_token() {
        let mut deps = mock_dependencies();
        let instantiate_msg = InstantiateMsg {
            owner:"creator".to_string(),
        };
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());
        
        let info = mock_info("creator", &[]);
        let msg = ExecuteMsg::AddCollection { members: vec![AdminInfo{
            address:"admin1".to_string(),
            amount:Uint128::new(5)
        },
        AdminInfo{
            address:"admin2".to_string(),
            amount:Uint128::new(15)
        }], 
        nft_address: "collection1".to_string(),
        collection: CollectionInfo { 
            total_nft:Uint128::new(2),
            check_mint:vec![1,2],
            url :"url".to_string(),
            image_url:"imag_url".to_string(),
            price:Uint128::new(20),
            denom : "ujunox".to_string(),
            max_nft:Uint128::new(1),
            mint_count:Uint128::new(0),
            name:"Collection1".to_string(),
            can_mint:true,
            is_rand_mint:true
            } 
        };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        query_collection_info(deps.as_ref(), "collection1".to_string()).unwrap();
        let user_info = query_user_info(deps.as_ref(), "collection1".to_string(), "user".to_string()).unwrap();
        assert_eq!(user_info,Uint128::new(0));

        let info = mock_info("creator", &[Coin{
            denom:"ujunox".to_string(),
            amount:Uint128::new(20)
        }]);
        let msg = ExecuteMsg::Mint { address: "collection1".to_string() };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

         let user_info = query_user_info(deps.as_ref(), "collection1".to_string(), "creator".to_string()).unwrap();
        assert_eq!(user_info,Uint128::new(1));

        let info = mock_info("creator1", &[Coin{
            denom:"ujunox".to_string(),
            amount:Uint128::new(20)
        }]);
        let msg = ExecuteMsg::Mint { address: "collection1".to_string() };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

       let collection = query_collection_info(deps.as_ref(), "collection1".to_string()).unwrap();
       let empty:Vec<u32> = vec![];
       assert_eq!(collection.check_mint,empty);

       let info = mock_info("creator", &[]);
       let msg = ExecuteMsg::AddCollection { members: vec![AdminInfo{
            address:"admin1".to_string(),
            amount:Uint128::new(5)
        },
        AdminInfo{
            address:"admin2".to_string(),
            amount:Uint128::new(15)
        }], 
        nft_address: "collection2".to_string(),
        collection: CollectionInfo { 
            total_nft:Uint128::new(1),
            check_mint:vec![1],
            url :"url".to_string(),
            image_url:"image_url".to_string(),
            price:Uint128::new(20),
            denom : "ujunox".to_string(),
            max_nft:Uint128::new(1),
            mint_count:Uint128::new(0),
            name:"Collection2".to_string(),
            can_mint:true,
            is_rand_mint:true
            } 
        };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        println!("{}","passed".to_string());

        let info = mock_info("creator", &[]);
       let msg = ExecuteMsg::SetMintFlag { address: "collection2".to_string(), flag: false };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("creator", &[]);
       let msg = ExecuteMsg::SetMintFlag { address: "collection2".to_string(), flag: true };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

       let info = mock_info("creator", &[Coin{
            denom:"ujunox".to_string(),
            amount:Uint128::new(20)
        }]);
        let msg = ExecuteMsg::Mint { address: "collection2".to_string() };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.messages.len(),3);

        assert_eq!(res.messages[0].msg,CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "collection2".to_string(),
            msg: to_binary(&Cw721BaseExecuteMsg::Mint(MintMsg {
                //::<Metadata>
                token_id: "Collection2.1".to_string(),
                owner: "creator".to_string(),
                token_uri: Some("url1.json".to_string()),
                extension:  Image{
                    image:Some("image_url1.png".to_string())
                }
            })).unwrap(),
            funds: vec![],
        }));
        
        assert_eq!(res.messages[1].msg,CosmosMsg::Bank(BankMsg::Send {
                to_address: "admin1".to_string(),
                amount:vec![Coin{
                    denom:"ujunox".to_string(),
                    amount:Uint128::new(5)
                }]
        }));
       
        assert_eq!(res.messages[2].msg,CosmosMsg::Bank(BankMsg::Send {
                to_address: "admin2".to_string(),
                amount:vec![Coin{
                    denom:"ujunox".to_string(),
                    amount:Uint128::new(15)
                }]
        }));

    }

}
