use {
    crate::{
        bridge_out::{
             instructions::bridge_to::BridgeToBumps, AdaptorID, BridgeTo, BridgeToArgs, SwapType
        },
        bridge_to_log::bridge_to_log,
        common::{XBridgeErrorCode, COMMISSION_DENOMINATOR, COMMISSION_RATE_LIMIT},
    }, 
    anchor_lang::{
        prelude::*, 
        solana_program::{program::invoke, system_instruction::transfer},
    }, 
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{self, Mint, Token, TokenAccount, Transfer},
        token_2022::Token2022,
        
    },
};


#[derive(Accounts)]
pub struct BridgeToSplCommission<'info> {
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

    #[account(
        mut,
        token::mint = mint,
    )]
    /// Commission Token Account.
    /// ATA: mint = mint
    /// This account is used to pay for commission token
    pub commission_token_account: Account<'info, TokenAccount>,

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
pub struct BridgeToCommissionArgs {
    pub adaptor_id: AdaptorID, // bridge adaptor id
    pub to: Vec<u8>,           // recipient address on target chain
    pub order_id: u64,         // order id for okx
    pub to_chain_id: u64,      // target chain id
    pub amount: u64,           // amount to bridge
    pub swap_type: SwapType,   // swap type
    pub data: Vec<u8>,         // data for bridge
    pub ext_data: Vec<u8>,     // ext data for extension feature

    pub commission_rate: u16,       // Commission rate
}

pub fn bridge_to_log_splcommission<'info>(
    ctx: Context<'_, '_, '_, 'info, BridgeToSplCommission<'info>>,
    data: BridgeToCommissionArgs,
) -> Result<()> {
    // commission
    require!(
        data.commission_rate > 0 && data.commission_rate <= COMMISSION_RATE_LIMIT,
        XBridgeErrorCode::InvalidCommissionRate
    );
    require!(
        ctx.accounts.commission_token_account.mint == ctx.accounts.mint.key(),
        XBridgeErrorCode::InvalidCommissionTokenAccount
    );
    let commission_amount = data
            .amount
            .checked_mul(data.commission_rate as u64)
            .ok_or(XBridgeErrorCode::CalculationError)?
            .checked_div(COMMISSION_DENOMINATOR - data.commission_rate as u64)
            .ok_or(XBridgeErrorCode::CalculationError)?;
    // transfer spl commission
    let cpi_commissionfee = Transfer {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.commission_token_account.to_account_info(),
        authority: ctx.accounts.payer.to_account_info(),
    };
    let cpi_ctx_commissionfee = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_commissionfee);
    token::transfer(cpi_ctx_commissionfee, commission_amount)?;
    msg!(
        "commission_to: {:?}, commission_amount: {:?}",
        ctx.accounts.commission_token_account.key(),
        commission_amount
    );


    // BridgeTo、BridgeToArgs
    let mut bridge_to_accounts = BridgeTo {
        payer: ctx.accounts.payer.clone(),
        user_token_account: ctx.accounts.user_token_account.clone(),
        mint: ctx.accounts.mint.clone(),
        associated_token_program: ctx.accounts.associated_token_program.clone(),
        token_program: ctx.accounts.token_program.clone(),
        token_2022_program: ctx.accounts.token_2022_program.clone(),
        system_program: ctx.accounts.system_program.clone(),
    };
    
    let _bridge_to_ctx: Context<'_, '_, '_, '_, BridgeTo<'_>> = Context::new(
        ctx.program_id,
        &mut bridge_to_accounts,
        ctx.remaining_accounts,
        BridgeToBumps::default(),
    );
    
    let _args = BridgeToArgs {
        adaptor_id: data.adaptor_id,
        to: data.to,
        order_id: data.order_id,
        to_chain_id: data.to_chain_id,
        amount: data.amount,
        swap_type: data.swap_type,
        data: data.data,
        ext_data: data.ext_data,
    };

    // xbirdge
    bridge_to_log(
        _bridge_to_ctx,
        _args,
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct BridgeToSolCommission<'info> {
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
    /// Commission Account.
    /// This account is used to pay for commission sol
    pub commission_account: SystemAccount<'info>,

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

pub fn bridge_to_log_solcommission<'info>(
    ctx: Context<'_, '_, '_, 'info, BridgeToSolCommission<'info>>,
    data: BridgeToCommissionArgs,
) -> Result<()> {
    // commission
    require!(
        data.commission_rate > 0 && data.commission_rate <= COMMISSION_RATE_LIMIT,
        XBridgeErrorCode::InvalidCommissionRate
    );
    let commission_amount = data
            .amount
            .checked_mul(data.commission_rate as u64)
            .ok_or(XBridgeErrorCode::CalculationError)?
            .checked_div(COMMISSION_DENOMINATOR - data.commission_rate as u64)
            .ok_or(XBridgeErrorCode::CalculationError)?;
    // transfer sol commission
    invoke(
        &transfer(&ctx.accounts.payer.key(), &ctx.accounts.commission_account.key(), commission_amount),
        &[
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.commission_account.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ]
    )?;
    msg!(
        "commission_to: {:?}, commission_amount: {:?}",
        ctx.accounts.commission_account.key(),
        commission_amount
    );

    // BridgeTo、BridgeToArgs
    let mut bridge_to_accounts = BridgeTo {
        payer: ctx.accounts.payer.clone(),
        user_token_account: ctx.accounts.user_token_account.clone(),
        mint: ctx.accounts.mint.clone(),
        associated_token_program: ctx.accounts.associated_token_program.clone(),
        token_program: ctx.accounts.token_program.clone(),
        token_2022_program: ctx.accounts.token_2022_program.clone(),
        system_program: ctx.accounts.system_program.clone(),
    };
    
    let _bridge_to_ctx: Context<'_, '_, '_, '_, BridgeTo<'_>> = Context::new(
        ctx.program_id,
        &mut bridge_to_accounts,
        ctx.remaining_accounts,
        BridgeToBumps::default(),
    );
    
    let _args = BridgeToArgs {
        adaptor_id: data.adaptor_id,
        to: data.to,
        order_id: data.order_id,
        to_chain_id: data.to_chain_id,
        amount: data.amount,
        swap_type: data.swap_type,
        data: data.data,
        ext_data: data.ext_data,
    };

    // xbirdge
    bridge_to_log(
        _bridge_to_ctx,
        _args,
    )?;

    Ok(())
}
