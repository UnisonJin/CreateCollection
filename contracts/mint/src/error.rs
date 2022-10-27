use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    
    #[error("Mint is not started yet")]
    MintNotStarted{},

    #[error("Mint is disabled by the admin")]
    MintDisabled{},

    #[error("All NFTS are minted")]
    SoldOut{},

    #[error("You are not white listed user")]
    NotWhiteListedUser{},

    #[error("You are exceeding your mint limit")]
    MaxPerAddressLimitExceeded{},

    #[error("Invalid reply ID")]
    InvalidReplyID {},
    
    #[error("Invalid content type")]
    InvalidContentType {},

    #[error("NFT collection contract initiate is failed")]
    InstantiateCw721Error {},


    #[error("InvalidNumTokens {max}, min: 1")]
    InvalidNumTokens { max: Uint128, min: Uint128 },

    #[error("Invalid minting limit per address. max: {max}, min: 1, got: {got}")]
    InvalidPerAddressLimit { max: Uint128, min: Uint128, got: Uint128 },

    #[error("InvalidDenom {expected} got {got}")]
    InvalidDenom { expected: String, got: String },

     #[error("Expected {price},sent {sent_money}")]
    NotExactFunds { price: Uint128, sent_money: Uint128 },

    #[error("InvalidStartTime {0} < {1}")]
    InvalidStartTime(u64, u64),

}
