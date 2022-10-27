#![cfg(test)]
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{ DepsMut, Empty, Decimal};

use cw721::{
    ContractInfoResponse, Cw721Query, CollectionInfo, Royalty
};

use crate::{
    Cw721Contract, Extension, InstantiateMsg
};

const MINTER: &str = "merlin";
const CONTRACT_NAME: &str = "Magic Power";
const SYMBOL: &str = "MGK";
const ADMIN: &str = "admin";

fn setup_contract(deps: DepsMut<'_>) -> Cw721Contract<'static, Extension, Empty> {
    let contract = Cw721Contract::default();
    let msg = InstantiateMsg {
        name: CONTRACT_NAME.to_string(),
        symbol: SYMBOL.to_string(),
        admin: String::from(ADMIN),
        collection_info: CollectionInfo{
            title: "title".to_string(),
            background_url: "background url".to_string(),
            logo_url: "logo_url".to_string(),
            description: "description".to_string(),
        },
        mint_info: None,
        royalty_info: Royalty { 
            address: "owner".to_string(),
            royalty_rate: Decimal::from_ratio(1 as u128, 10 as u128) 
        },
         minter:Some("minter".to_string())
    };
    let info = mock_info("creator", &[]);
    let res = contract.instantiate(deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    contract
}

#[test]
fn proper_instantiation() {
    let mut deps = mock_dependencies();
    let contract = Cw721Contract::<Extension, Empty>::default();

    let msg = InstantiateMsg {
        name: CONTRACT_NAME.to_string(),
        symbol: SYMBOL.to_string(),
        admin: String::from(ADMIN),
        collection_info:CollectionInfo{
            title: "title".to_string(),
            background_url: "background url".to_string(),
            logo_url: "logo_url".to_string(),
            description: "description".to_string(),
        },
        mint_info:None,
        royalty_info: Royalty { 
            address: "owner".to_string(),
            royalty_rate: Decimal::from_ratio(1 as u128, 10 as u128) 
        },
        minter:Some("minter".to_string())
    };
    let info = mock_info("creator", &[]);

    // we can just call .unwrap() to assert this was a success
    let res = contract
        .instantiate(deps.as_mut(), mock_env(), info, msg)
        .unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = contract.minter(deps.as_ref()).unwrap();
    assert_eq!(MINTER, res.minter);

    let res = contract.admin(deps.as_ref()).unwrap();
    assert_eq!(ADMIN, res.admin);

    let info = contract.contract_info(deps.as_ref()).unwrap();
    assert_eq!(
        info,
        ContractInfoResponse {
            name: CONTRACT_NAME.to_string(),
            symbol: SYMBOL.to_string(),
        }
    );

    let count = contract.num_tokens(deps.as_ref()).unwrap();
    assert_eq!(0, count.count);

    // list the token_ids
    //let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();
    // assert_eq!(0, tokens.tokens.len());
}
