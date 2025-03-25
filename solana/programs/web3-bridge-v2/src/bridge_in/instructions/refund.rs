use {
    crate::{
        bridge_in::{
            ContractConfig,
            BridgeMessage,
            ToSwapMessageState,
        },
        common::{
            XBridgeErrorCode,
            TEST_MPC_STR,
            TEST_MPC_STR_2
        }
    },
    anchor_lang::{
        prelude::*,
        solana_program::
            system_program
        ,
    },
    anchor_spl::{
        token::{self, spl_token, Transfer},
        associated_token::get_associated_token_address,
        token_interface::{TokenInterface, TokenAccount, Mint}
    },
};


#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(signer)]
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
    )]
    /// CHECK: This is the refund account for the to address's from_token ATA
    pub refund_token_account: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = source_mint,
        token::authority = gasrefund,
    )]
    pub gasrefund_source_token_account: InterfaceAccount<'info, TokenAccount>,

    pub source_mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    /// CHECK: 
    pub toswap_message_request: Box<Account<'info, ToSwapMessageState>>,

    /// SPL Token Program
    /// "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    /// CHECK: fixed
    #[account(address = spl_token::id())]
    pub token_program: Interface<'info, TokenInterface>,

    #[account(
        constraint = authorized_caller.key() == contract_config.mpc ||
                    authorized_caller.key().to_string() == TEST_MPC_STR ||
                    authorized_caller.key().to_string() == TEST_MPC_STR_2 @ XBridgeErrorCode::Unauthorized,
        constraint = !contract_config.paused @ XBridgeErrorCode::AlreadyPaused,
        seeds = [b"contract_config"],
        bump
    )]
    /// CHECK:
    pub contract_config: Account<'info, ContractConfig>,

    /// System Program.
    /// "11111111111111111111111111111111"
    /// CHECK: fixed
    #[account(address = system_program::id())]
    pub system_program: Program<'info, System>,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct RefundArgs {
    pub refund_amount: u64,
    pub create_pda_fee: u64,
    pub orderid: u128,
}

pub fn refund<'info>(
    _ctx: Context<'_, '_, '_, 'info, Refund<'info>>, 
    data: RefundArgs,
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
    // verify (refund_address & source_mint)  == _ctx.accounts.refund_token_account
    let refund_address_bytes = request_src_chain_message.to;
    let refund_address = Pubkey::from(refund_address_bytes);
    require!(
        get_associated_token_address(&refund_address, &_ctx.accounts.source_mint.key()) == _ctx.accounts.refund_token_account.key(),
        XBridgeErrorCode::InvalidDexSwapArgsRefundAddress
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
    let total_amount = data.refund_amount.checked_add(data.create_pda_fee).expect("Addition overflow");
    require!(
        max_amount >= total_amount,
        XBridgeErrorCode::InvalidDexSwapArgsFromAmount
    );

    let authority_seeds: &[&[u8]] = &[b"xbridge_authority_pda", &[_ctx.bumps.xbridge_authority]];
    let signer_seeds = [
            &authority_seeds[..]
    ];

    let token_program = _ctx.accounts.token_program.to_account_info();
    let source_token_account = _ctx.accounts.xbridge_source_token_account.to_account_info();
    // refund 
    if data.refund_amount > 0 {
        let user_token_account = _ctx.accounts.refund_token_account.to_account_info();
        let cpi_accounts = Transfer {
            from: source_token_account.clone(),
            to: user_token_account.clone(),
            authority: _ctx.accounts.xbridge_authority.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(token_program.clone(), cpi_accounts, &signer_seeds);
        token::transfer(cpi_ctx.with_signer(&[authority_seeds]), data.refund_amount)?;
    }

    // sending transaction fee to gasrefund ata address
    if data.create_pda_fee > 0 {
        let gasrefund_token_account = _ctx.accounts.gasrefund_source_token_account.to_account_info();
        let cpi_accounts_fee = Transfer {
            from: source_token_account.clone(),
            to: gasrefund_token_account.clone(),
            authority: _ctx.accounts.xbridge_authority.to_account_info(),
        };
        let cpi_ctx_fee = CpiContext::new_with_signer(token_program.clone(), cpi_accounts_fee, &signer_seeds);
        token::transfer(cpi_ctx_fee.with_signer(&[authority_seeds]), data.create_pda_fee)?;
    }

    toswap_message_request.is_used = true;

    Ok(())
}