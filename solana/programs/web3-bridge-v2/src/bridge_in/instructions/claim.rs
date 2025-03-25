use {
    crate::{
        bridge_in::{
            BridgeMessage, ContractConfig, ToSwapMessageState
        },
        common::{
            dexrouter_program, 
            wrapped_sol, 
            XBridgeErrorCode, 
            TEST_MPC_STR, TEST_MPC_STR_2
        },
    },
    anchor_lang::{
        prelude::*,
        solana_program::{program_pack::Pack, system_program}
    },
    anchor_spl::{
        associated_token::{get_associated_token_address, get_associated_token_address_with_program_id}, token::{self, spl_token, Transfer}, token_2022::Token2022, token_interface::{Mint, TokenAccount, TokenInterface}
    },
    dex_solana::{cpi::accounts::SwapAccounts, SwapArgs},
};


#[derive(Accounts)]
pub struct Claim<'info> {
    #[account(
        signer
    )]
    /// The authorized caller.
    /// CHECK: The address authorized to call the claim function.
    pub authorized_caller: Signer<'info>,

    /// The authority PDA derived from xbridge_program
    /// CHECK: This account is the authority derived from the xbridge_program.
    #[account(
        seeds = [b"xbridge_authority_pda"],
        bump,
    )]
    pub xbridge_authority: AccountInfo<'info>,

    /// CHECK: gasrefund
    pub gasrefund: AccountInfo<'info>,

    #[account(
        mut,
        token::mint = source_mint,
        token::authority = xbridge_authority,
    )]
    pub xbridge_source_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = source_mint,
        token::authority = gasrefund,
    )]
    pub gasrefund_source_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = destination_mint,
        constraint = (destination_token_account.to_account_info().owner == &spl_token::id()) || (destination_token_account.to_account_info().owner == &Token2022::id()) @ XBridgeErrorCode::AccountOwnedByWrongProgram
    )]
    pub destination_token_account: InterfaceAccount<'info, TokenAccount>,

    pub source_mint: InterfaceAccount<'info, Mint>,

    pub destination_mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub toswap_message_request: Box<Account<'info, ToSwapMessageState>>,

    #[account(address = dexrouter_program::ID)]
    /// CHECK: dex_program
    pub dex_program: AccountInfo<'info>,

    #[account(
        constraint = authorized_caller.key() == contract_config.mpc ||
                    authorized_caller.key().to_string() == TEST_MPC_STR ||
                    authorized_caller.key().to_string() == TEST_MPC_STR_2 @ XBridgeErrorCode::Unauthorized,
        constraint = !contract_config.paused @ XBridgeErrorCode::AlreadyPaused,
        seeds = [b"contract_config"],
        bump
    )]
    pub contract_config: Account<'info, ContractConfig>,

    /// SPL Token Program
    /// "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    /// CHECK: fixed
    #[account(address = spl_token::id())]
    pub token_program: Interface<'info, TokenInterface>,

    /// System Program.
    /// "11111111111111111111111111111111"
    /// CHECK: fixed
    #[account(address = system_program::id())]
    pub system_program: Program<'info, System>,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct ClaimArgs {
    pub create_pda_fee: u64,
    pub dex_swap_args: SwapArgs,
    pub orderid: u128,
}

pub fn claim<'info>(
    _ctx: Context<'_, '_, '_, 'info, Claim<'info>>, 
    data: ClaimArgs,
) -> Result<()> {
    // toswap_message_request.is_used != true
    let toswap_message_request = &mut _ctx.accounts.toswap_message_request;
    require!(
        !toswap_message_request.is_used,
        XBridgeErrorCode::ToswapAlreadyUsed
    );
    
    // read toswap_message_request.data
    let request_src_chain_message = BridgeMessage::try_from_slice(&toswap_message_request.data)?;
    BridgeMessage::msg_oracle_data(&request_src_chain_message, data.orderid);
    // verify (to_address & destination_mint)  == _ctx.accounts.destination_token_account
    let to_address_bytes = request_src_chain_message.to;
    let to_address = Pubkey::from(to_address_bytes);
    require!(
        // spl-token or spl-2022-token
        (get_associated_token_address(&to_address, &_ctx.accounts.destination_mint.key()) == _ctx.accounts.destination_token_account.key()) || (get_associated_token_address_with_program_id(&to_address, &_ctx.accounts.destination_mint.key(), &Token2022::id()) == _ctx.accounts.destination_token_account.key()),
        XBridgeErrorCode::InvalidDexSwapArgsToAddress
    );
    // verify from_token_address == _ctx.accounts.source_mint
    let from_token_bytes = request_src_chain_message.from_token;
    let from_token_address = Pubkey::from(from_token_bytes);
    require!(
        from_token_address == _ctx.accounts.source_mint.key(),
        XBridgeErrorCode::InvalidDexSwapArgsFromTokenAddress
    );
    // verify from_amount == data.dex_swap_args.amount_in + data.create_pda_fee
    let from_amount_bytes = &request_src_chain_message.from_amount[24..32];
    let from_amount = u64::from_be_bytes(from_amount_bytes.try_into().expect("slice with incorrect length"));
    let max_amount = from_amount.checked_mul(11).and_then(|v| v.checked_div(10)).expect("Multiplication overflow");
    let total_amount = data.dex_swap_args.amount_in.checked_add(data.create_pda_fee).expect("Addition overflow");
    require!(
        max_amount >= total_amount,
        XBridgeErrorCode::InvalidDexSwapArgsFromAmount
    );

    let authority_seeds: &[&[u8]] = &[b"xbridge_authority_pda", &[_ctx.bumps.xbridge_authority]];
    let signer_seeds = [
            &authority_seeds[..]
    ];

    let dex_remaining_accounts = _ctx.remaining_accounts.to_vec();
    let dex_swap_args = data.dex_swap_args.clone();      
    let dex_swap_ctx = CpiContext::new_with_signer(
        _ctx.accounts.dex_program.to_account_info(), 
        SwapAccounts {
            payer: _ctx.accounts.xbridge_authority.to_account_info(),
            source_token_account: _ctx.accounts.xbridge_source_token_account.to_account_info(),
            destination_token_account: _ctx.accounts.destination_token_account.to_account_info(),
            source_mint: _ctx.accounts.source_mint.to_account_info(),
            destination_mint: _ctx.accounts.destination_mint.to_account_info(),
        },
        &signer_seeds
    )
    .with_remaining_accounts(dex_remaining_accounts);
    dex_solana::cpi::swap(dex_swap_ctx, dex_swap_args, 0)?;

    // sending transaction fee to gasrefund ata address
    if data.create_pda_fee > 0 {
        let token_program = _ctx.accounts.token_program.to_account_info();
        let xbridge_source_token_account = _ctx.accounts.xbridge_source_token_account.to_account_info();
        let gasrefund_token_account = _ctx.accounts.gasrefund_source_token_account.to_account_info();
        let cpi_accounts = Transfer {
            from: xbridge_source_token_account.clone(),
            to: gasrefund_token_account.clone(),
            authority: _ctx.accounts.xbridge_authority.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(token_program.clone(), cpi_accounts, &signer_seeds);
        token::transfer(cpi_ctx, data.create_pda_fee)?;
    }

    toswap_message_request.is_used = true;

    Ok(())
}

pub fn claim_to_sol<'info>(
    _ctx: Context<'_, '_, '_, 'info, Claim<'info>>, 
    data: ClaimArgs,
) -> Result<()> {
    // toswap_message_request.is_used != true
    let toswap_message_request = &mut _ctx.accounts.toswap_message_request;
    require!(
        !toswap_message_request.is_used,
        XBridgeErrorCode::ToswapAlreadyUsed
    );
    
    // read toswap_message_request.data
    let request_src_chain_message = BridgeMessage::try_from_slice(&toswap_message_request.data)?;
    BridgeMessage::msg_oracle_data(&request_src_chain_message, data.orderid);
    // verify (to_address & destination_mint)  == _ctx.accounts.destination_token_account
    let destination_account_info = &_ctx.accounts.destination_token_account.to_account_info();
    let token_account = spl_token::state::Account::unpack(&destination_account_info.try_borrow_data()?)
        .map_err(|_| XBridgeErrorCode::WsolPdaFailedToDecodeTokenAccount)?;
    require!(
        token_account.mint == wrapped_sol::ID,      // Must be WSOL SPL-PDA
        XBridgeErrorCode::WsolPdaInvalidMintAddress
    );
    require!(
        token_account.owner == _ctx.accounts.contract_config.mpc ||
        token_account.owner.to_string() == TEST_MPC_STR,       // Must be created by the mpc
        XBridgeErrorCode::WsolPdaInvalidOwnerAddress
    );
    require!(
        token_account.amount == 0,          // The WSOL SPL-PDA must be a new account with a balance of 0
        XBridgeErrorCode::WsolPdaInvalidAccountAmount
    );
    // verify from_token_address == _ctx.accounts.source_mint
    let from_token_bytes = request_src_chain_message.from_token;
    let from_token_address = Pubkey::from(from_token_bytes);
    require!(
        from_token_address == _ctx.accounts.source_mint.key(),
        XBridgeErrorCode::InvalidDexSwapArgsFromTokenAddress
    );
    // verify from_amount == data.dex_swap_args.amount_in + data.create_pda_fee
    let from_amount_bytes = &request_src_chain_message.from_amount[24..32];
    let from_amount = u64::from_be_bytes(from_amount_bytes.try_into().expect("slice with incorrect length"));
    let max_amount = from_amount.checked_mul(11).and_then(|v| v.checked_div(10)).expect("Multiplication overflow");
    let total_amount = data.dex_swap_args.amount_in.checked_add(data.create_pda_fee).expect("Addition overflow");
    require!(
        max_amount >= total_amount,
        XBridgeErrorCode::InvalidDexSwapArgsFromAmount
    );

    let authority_seeds: &[&[u8]] = &[b"xbridge_authority_pda", &[_ctx.bumps.xbridge_authority]];
    let signer_seeds = [
            &authority_seeds[..]
    ];

    let dex_remaining_accounts = _ctx.remaining_accounts.to_vec();
    let dex_swap_args = data.dex_swap_args.clone();      
    let dex_swap_ctx = CpiContext::new_with_signer(
        _ctx.accounts.dex_program.to_account_info(), 
        SwapAccounts {
            payer: _ctx.accounts.xbridge_authority.to_account_info(),
            source_token_account: _ctx.accounts.xbridge_source_token_account.to_account_info(),
            destination_token_account: _ctx.accounts.destination_token_account.to_account_info(),
            source_mint: _ctx.accounts.source_mint.to_account_info(),
            destination_mint: _ctx.accounts.destination_mint.to_account_info(),
        },
        &signer_seeds
    )
    .with_remaining_accounts(dex_remaining_accounts);
    dex_solana::cpi::swap(dex_swap_ctx, dex_swap_args, 0)?;

    // sending transaction fee to gasrefund ata address
    if data.create_pda_fee > 0 {
        let token_program = _ctx.accounts.token_program.to_account_info();
        let xbridge_source_token_account = _ctx.accounts.xbridge_source_token_account.to_account_info();
        let gasrefund_token_account = _ctx.accounts.gasrefund_source_token_account.to_account_info();
        let cpi_accounts = Transfer {
            from: xbridge_source_token_account.clone(),
            to: gasrefund_token_account.clone(),
            authority: _ctx.accounts.xbridge_authority.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(token_program.clone(), cpi_accounts, &signer_seeds);
        token::transfer(cpi_ctx, data.create_pda_fee)?;
    }

    toswap_message_request.is_used = true;

    Ok(())
}