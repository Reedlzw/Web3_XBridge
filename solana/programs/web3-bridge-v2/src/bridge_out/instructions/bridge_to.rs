use {
    crate::
        bridge_out::{
            AdaptorID, SwapType
        }
    , anchor_lang::prelude::*, anchor_spl::{
        associated_token::AssociatedToken,
        token::{Mint, Token, TokenAccount},
        token_2022::Token2022,
    },
};

#[derive(Accounts)]
pub struct BridgeTo<'info> {
    #[account(mut)]
    /// Payer.
    /// This account is user to pay for the transaction and bridge token.
    /// CHECK: This account is user to pay for the transaction and bridge token.
    pub payer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = payer,
    )]
    /// User Token Account.
    /// ATA: mint = mint, owner = payer,
    /// This account is used to pay for bridge token, owner by payer(user).
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    /// Mint.
    /// This account is used to check the mint of source_token_account.
    pub mint: Account<'info, Mint>,

    /// Associated Token Program
    /// "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
    /// CHECK: fixed
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// SPL Token Program
    /// "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    /// CHECK: fixed
    #[account(address = Token::id())]
    pub token_program: AccountInfo<'info>,

    /// SPL Token 2022 Program
    /// "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
    /// CHECK: fixed
    #[account(address = Token2022::id())]
    pub token_2022_program: AccountInfo<'info>,

    /// System Program.
    /// "11111111111111111111111111111111"
    /// CHECK: fixed
    #[account(address = System::id())]
    pub system_program: AccountInfo<'info>,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct BridgeToArgs {
    pub adaptor_id: AdaptorID, // bridge adaptor id
    pub to: Vec<u8>,           // recipient address on target chain
    pub order_id: u64,         // order id for okx
    pub to_chain_id: u64,      // target chain id
    pub amount: u64,           // amount to bridge
    pub swap_type: SwapType,   // swap type
    pub data: Vec<u8>,         // data for bridge
    pub ext_data: Vec<u8>,     // ext data for extension feature
}