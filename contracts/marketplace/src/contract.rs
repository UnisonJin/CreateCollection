use cosmwasm_std::{
    coin, entry_point, from_binary, to_binary, BankMsg, Binary, CosmosMsg, Decimal, Deps, DepsMut,
    Env, MessageInfo, Order, Response, StdResult, Uint128, WasmMsg,
};

use crate::cw721::{Cw721ExecuteMsg, Cw721ReceiveMsg};
use crate::error::ContractError;
use crate::msg::{
     ExecuteMsg, FeeResponse, InstantiateMsg, Offer, OffersResponse, QueryMsg,
    SellNft,
};
use crate::state::{get_fund, increment_offerings, Offering, State, OFFERINGS, STATE, CollectionInfo, COLLECTIONINFO, SALEHISTORY,SaleHistoryInfo};
use cw2::set_contract_version;
use cw_storage_plus::Bound;
use std::ops::{Mul, Sub};

// version info for migration info
const CONTRACT_NAME: &str = "human_market_place";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let state = State {
        fee: msg.fee,
        owner: info.sender,
        tvl: Uint128::zero()
    };
    STATE.save(deps.storage, &state)?;

    Ok(Response::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Buy { offering_id,address } => execute_buy(deps,env, info, offering_id,address),
        ExecuteMsg::WithdrawNft { offering_id,address } => execute_withdraw(deps, info, offering_id,address),
        ExecuteMsg::ReceiveNft(msg) => execute_receive_nft(deps, info, msg),
        ExecuteMsg::WithdrawFees { amount, denom } => {
            execute_withdraw_fees(deps, info, amount, denom)
        }
        ExecuteMsg::ChangeFee { fee } => execute_change_fee(deps, info, fee),
        ExecuteMsg::AddCollection { address } => execute_add_collection(deps, info, address)
    }
}

pub fn execute_buy(
    deps: DepsMut,
    env:Env,
    info: MessageInfo,
    offering_id: String,
    address:String
) -> Result<Response, ContractError> {
    // check if offering exists
    let collection_info =  is_registered_collection(deps.as_ref(), &address)?;
    let off = OFFERINGS.load(deps.storage, (&address,&offering_id))?;

    if off.seller.eq(&info.sender) {
        return Err(ContractError::InvalidBuyer {});
    }

    // check for enough coins
    let off_fund = get_fund(info.funds.clone(), off.list_price.denom.clone())?;
    if off_fund.amount != off.list_price.amount.clone() {
        return Err(ContractError::InsufficientFunds {});
    }

    let state = STATE.load(deps.storage)?;
    let net_amount = Decimal::one().sub(state.fee).mul(off_fund.amount);
    // create transfer msg
    let transfer_msg: CosmosMsg = BankMsg::Send {
        to_address: off.seller.clone().into(),
        amount: vec![coin(net_amount.u128(), off_fund.denom.clone())],
    }
    .into();

    //tvl update for all collection
    STATE.update(deps.storage, |mut state| -> StdResult<_>{
        state.tvl = state.tvl + off_fund.amount;
        Ok(state)
    })?;

    //update sale history of this collection
    SALEHISTORY.save(deps.storage, (&address,&(collection_info.sale_id+1).to_string()),&SaleHistoryInfo { 
        from:off.seller.clone().to_string(),
        to: info.sender.to_string(), 
        denom: off.list_price.denom.clone(),
        amount: off_fund.amount, 
        time: env.block.time.seconds(),
        nft_address:address.clone(),
        token_id:off.token_id.clone()
    })?;

    //udpate tvl and sale id for this collection
    COLLECTIONINFO.update(deps.storage, &address, 
    |collection_info|->StdResult<_>{
        let mut collection_info = collection_info.unwrap();
        collection_info.sale_id += 1;
        collection_info.tvl = collection_info.tvl + off.list_price.amount;
        Ok(collection_info)
    })?;

    // create transfer cw721 msg
    let cw721_transfer = Cw721ExecuteMsg::TransferNft {
        recipient: info.sender.clone().into(),
        token_id: off.token_id.clone(),
    };
    let cw721_transfer_msg: CosmosMsg = WasmMsg::Execute {
        contract_addr: off.contract.clone().into(),
        msg: to_binary(&cw721_transfer)?,
        funds: vec![],
    }
    .into();

    OFFERINGS.remove(deps.storage, (&address,&offering_id));

    let price_string = format!("{}{}", off_fund.amount, off_fund.denom);
    let res = Response::new()
        .add_attribute("action", "buy_nft")
        .add_attribute("buyer", info.sender)
        .add_attribute("seller", off.seller)
        .add_attribute("paid_price", price_string)
        .add_attribute("token_id", off.token_id)
        .add_attribute("nft_contract", off.contract)
        .add_messages(vec![transfer_msg, cw721_transfer_msg]);
    Ok(res)
}

pub fn execute_withdraw(
    deps: DepsMut,
    info: MessageInfo,
    offering_id: String,
    address: String
) -> Result<Response, ContractError> {
    let off = OFFERINGS.load(deps.storage, (&address,&offering_id))?;
    if off.seller.ne(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let transfer_cw721_msg = Cw721ExecuteMsg::TransferNft {
        recipient: off.seller.into(),
        token_id: off.token_id.clone(),
    };

    let exec_cw721_transfer: CosmosMsg = WasmMsg::Execute {
        contract_addr: off.contract.into(),
        msg: to_binary(&transfer_cw721_msg)?,
        funds: vec![],
    }
    .into();

    OFFERINGS.remove(deps.storage, (&address,&offering_id));

    let res = Response::new()
        .add_attribute("action", "withdraw_nft")
        .add_attribute("seller", info.sender)
        .add_message(exec_cw721_transfer);
    Ok(res)
}

pub fn execute_receive_nft(
    deps: DepsMut,
    info: MessageInfo,
    wrapper: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    let msg: SellNft = from_binary(&wrapper.msg)?;
    let nft_address = info.sender.to_string();

    let id = increment_offerings(deps.storage,nft_address.clone())?.to_string();

    is_registered_collection(deps.as_ref(), &nft_address)?;

    // save Offering
    let off = Offering {
        contract: info.sender.clone(),
        token_id: wrapper.token_id,
        seller: deps.api.addr_validate(&wrapper.sender)?,
        list_price: msg.list_price.clone(),
    };
    OFFERINGS.save(deps.storage, (&id,&nft_address), &off)?;

    let price_string = format!("{}{}", msg.list_price.amount, msg.list_price.denom);
    let res = Response::new()
        .add_attribute("action", "sell_nft")
        .add_attribute("offering_id", id)
        .add_attribute("nft_contract", info.sender)
        .add_attribute("seller", off.seller)
        .add_attribute("list_price", price_string)
        .add_attribute("token_id", off.token_id);
    Ok(res)
}

pub fn execute_withdraw_fees(
    deps: DepsMut,
    info: MessageInfo,
    amount: Uint128,
    denom: String,
) -> Result<Response, ContractError> {
    let state = STATE.load(deps.storage)?;

    if state.owner.ne(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let transfer: CosmosMsg = BankMsg::Send {
        to_address: state.owner.into(),
        amount: vec![coin(amount.into(), denom)],
    }
    .into();

    Ok(Response::new().add_message(transfer))
}

pub fn execute_change_fee(
    deps: DepsMut,
    info: MessageInfo,
    fee: Decimal,
) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if state.owner.ne(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }

        state.fee = fee;
        Ok(state)
    })?;

    let res = Response::new()
        .add_attribute("action", "change_fee")
        .add_attribute("fee", fee.to_string());
    Ok(res)
}

pub fn execute_add_collection(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {

    let state = STATE.load(deps.storage)?;
    
    if state.owner.ne(&info.sender) {
            return Err(ContractError::Unauthorized {});
        }

    COLLECTIONINFO.save(deps.storage, &address, &CollectionInfo {
         sale_id:0 , 
         tvl: Uint128::new(0),
         num_offerings: 0 })?;
 
    let res = Response::new()
        .add_attribute("action", "register collection")
        .add_attribute("address", address);
    Ok(res)
}


pub fn is_registered_collection(
    deps:Deps,
    address:&String
)->Result<CollectionInfo,ContractError>{
    let collection_info = COLLECTIONINFO.may_load(deps.storage, address)?;
    if collection_info == None{
        return Err(ContractError::NotListedNFt{});
    }
    else{
        Ok(collection_info.unwrap())
    }
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetStateInfo{} => to_binary(&query_state(deps)?),
        QueryMsg::GetCollectionInfo { address } => to_binary(&query_collection_info(deps,address)?),
        QueryMsg::GetFee {} => to_binary(&query_fee(deps)?),
        QueryMsg::GetOffers { start_after, limit, address }  => {
            to_binary(&query_all(deps, start_after, limit,address)?)
        }
    }
}

fn query_state(deps: Deps) -> StdResult<State> {
    let state = STATE.load(deps.storage)?;
    Ok(state)
}

fn query_collection_info(deps: Deps, address: String) -> StdResult<CollectionInfo>{
    let collection_info = is_registered_collection(deps, &address).unwrap();
    Ok(collection_info)
}

fn query_fee(deps: Deps) -> StdResult<FeeResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(FeeResponse { fee: state.fee })
}

fn query_all(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
    address:String
) -> StdResult<OffersResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let offers: StdResult<Vec<Offer>> = OFFERINGS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| item.map(map_offer))
        .collect();

    Ok(OffersResponse { offers: offers? })
}

fn map_offer((k, v): (Vec<u8>, Offering)) -> Offer {
    Offer {
        id: String::from_utf8_lossy(&k).to_string(),
        token_id: v.token_id,
        contract: v.contract,
        seller: v.seller,
        list_price: v.list_price,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_dependencies_with_balance, mock_env, mock_info,
    };
    use cosmwasm_std::{coins, Decimal, SubMsg};

    fn setup(deps: DepsMut) {
        let msg = InstantiateMsg {
            fee: Decimal::percent(2),
        };
        let info = mock_info("creator", &[]);

        // we can just call .unwrap() to assert this was a success
        instantiate(deps, mock_env(), info, msg).unwrap();
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            fee: Decimal::percent(2),
        };
        let info = mock_info("creator", &[]);

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn sell_nft() {
        let mut deps = mock_dependencies();
        setup(deps.as_mut());

        let sell_msg = SellNft {
            list_price: coin(1000, "earth"),
        };

        let msg = ExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
            token_id: "1".into(),
            sender: "owner".into(),
            msg: to_binary(&sell_msg).unwrap(),
        });
        let info = mock_info("nft-collectibles", &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(0, res.messages.len());

        let msg = QueryMsg::GetOffers {
            start_after: None,
            limit: None,
            address:"collection1".to_string()
        };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let value: OffersResponse = from_binary(&res).unwrap();

        assert_eq!("1", value.offers.first().unwrap().token_id);
    }

    #[test]
    fn buy_nft() {
        let mut deps = mock_dependencies();
        setup(deps.as_mut());

        let sell_msg = SellNft {
            list_price: coin(1000, "earth"),
        };

        let msg = ExecuteMsg::ReceiveNft(Cw721ReceiveMsg {
            token_id: "1".into(),
            sender: "owner".into(),
            msg: to_binary(&sell_msg).unwrap(),
        });
        let info = mock_info("nft-collectibles", &[]);
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg.clone()).unwrap();
        assert_eq!(0, res.messages.len());

        let msg = ExecuteMsg::Buy {
            offering_id: "1".into(),
            address :"collection1".to_string()
        };
        let info = mock_info("owner", &coins(1000, "earth"));
        let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
        match res {
            Err(ContractError::InvalidBuyer {}) => {}
            _ => panic!("Must return InvalidBuyer error"),
        }

        let info = mock_info("anyone", &coins(400, "earth"));
        let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
        match res {
            Err(ContractError::InsufficientFunds {}) => {}
            _ => panic!("Must return InsufficientFunds error"),
        }

        let info = mock_info("anyone", &coins(1000, "earth"));
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(2, res.messages.len());
        assert_eq!(
            res.messages[0],
            SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: "owner".into(),
                amount: coins(980, "earth")
            }))
        );
    }

    #[test]
    fn withdraw_fees() {
        let mut deps = mock_dependencies_with_balance(&coins(1000, "earth"));
        setup(deps.as_mut());

        let msg = ExecuteMsg::WithdrawFees {
            amount: 1000u32.into(),
            denom: "earth".into(),
        };
        let info = mock_info("anyone", &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return Unauthorized error"),
        }

        let info = mock_info("creator", &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(1, res.messages.len());
        assert_eq!(
            res.messages[0],
            SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: "creator".into(),
                amount: coins(1000, "earth")
            }))
        );
    }

    #[test]
    fn change_fee() {
        let mut deps = mock_dependencies();
        setup(deps.as_mut());

        let msg = ExecuteMsg::ChangeFee {
            fee: Decimal::percent(3),
        };
        let info = mock_info("anyone", &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return Unauthorized error"),
        }

        let info = mock_info("creator", &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let msg = QueryMsg::GetFee {};
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let value: FeeResponse = from_binary(&res).unwrap();
        assert_eq!(Decimal::percent(3), value.fee);
    }
}
