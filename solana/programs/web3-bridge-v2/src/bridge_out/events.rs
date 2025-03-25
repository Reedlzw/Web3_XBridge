use {
    anchor_lang::prelude::*,
    serde::{Deserialize, Serialize},
};

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Serialize, Deserialize)]
pub struct LogBridgeTo {
    pub order_id: String,
    pub adaptor_id: u8,
    pub to: String,
    pub amount: u64,
    pub swap_type: u8,
    pub to_chain_id: u8,
    pub bridge_token: String,
    pub ext: String,
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Serialize, Deserialize)]
pub struct LogBridgeToVersion1 {
    pub order_id: String,
    pub adaptor_id: u8,
    pub to: String,     // 桥出的地址，桥给谁打钱，给xbridge或者给用户
    pub amount: u64,
    pub swap_type: u8,
    pub to_chain_id: u64,
    pub bridge_token: String,
    pub src_chain_id: u16,
    pub from: String,        // solana链上发起用户的地址
    pub user_address: String,     // toswap接受钱包地址
    pub ext: String,
}

#[event]
pub struct LogBridgeToVersion1Event {
    pub order_id: String,
    pub adaptor_id: u8,
    pub to: String,     // 桥出的地址，桥给谁打钱，给xbridge或者给用户
    pub amount: u64,
    pub swap_type: u8,
    pub to_chain_id: u64,
    pub bridge_token: String,
    pub src_chain_id: u16,
    pub from: String,        // solana链上发起用户的地址
    pub user_address: String,     // toswap接受钱包地址
    pub ext: String,
}