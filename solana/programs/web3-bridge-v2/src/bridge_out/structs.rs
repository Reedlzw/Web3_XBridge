use {
    anchor_lang::prelude::*,
    serde::{Deserialize, Serialize},
};

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Serialize, Deserialize)]
pub struct BridgeToArgsExtData {
    pub user_address: Vec<u8>,     // toswap接受钱包地址
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BridgeResult{
    pub ext: String,
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Serialize, Deserialize)]
pub struct RelayerFee{
    pub amount: u64,
    pub mint: String,
    pub to: String,
}

#[derive(Debug, PartialEq, AnchorDeserialize, AnchorSerialize, Clone, Copy)]
pub enum SwapType {
    BRIDGE,
    SWAPANDBRIDGE,
}

#[derive(Debug, PartialEq, AnchorDeserialize, AnchorSerialize, Clone, Copy)]
pub enum AdaptorID {
    /* 00 */ Bridge0,
    /* 01 */ Bridge1,
    /* 02 */ Bridge2,
    /* 03 */ Bridgers,
    /* 04 */ Bridge4,
    /* 05 */ Bridge5,
    /* 06 */ Bridge6,
    /* 07 */ Bridge7,
    /* 08 */ Bridge8,
    /* 09 */ Bridge9,
    /* 10 */ Bridge10,
    /* 11 */ Bridge11,
    /* 12 */ Bridge12,
    /* 13 */ Bridge13,
    /* 14 */ Bridge14,
    /* 15 */ Bridge15,
    /* 16 */ Bridge16,
    /* 17 */ Wanchain,
    /* 18 */ Cctp,
    /* 19 */ Bridge19,
    /* 20 */ Bridge20,
    /* 21 */ Wormhole,
    /* 22 */ Meson,
    /* 23 */ Bridge23,
    /* 24 */ Bridge24,
    /* 25 */ Bridge25,
    /* 26 */ Bridge26,
    /* 27 */ Bridge27,
    /* 28 */ Bridge28,
    /* 29 */ Bridge29,
    /* 30 */ Bridge30,
    /* 31 */ Bridge31,
    /* 32 */ Bridge32,
    /* 33 */ Bridge33,
    /* 34 */ Debridgedln,
    /* 35 */ Bridge35,
    /* 36 */ Bridge36,
    /* 37 */ Bridge37,
    /* 38 */ Bridge38,
    /* 39 */ Bridge39,
    /* 40 */ Bridge40,
    /* 41 */ Allbridge,
    /* 42 */ Bridge42,
    /* 43 */ Bridge43,
    /* 44 */ Bridge44,
    /* 45 */ Bridge45,
    /* 46 */ Bridge46,
    /* 47 */ MayanSwift,
    /* 48 */ Bridge48,
    /* 49 */ Bridge49,
}
