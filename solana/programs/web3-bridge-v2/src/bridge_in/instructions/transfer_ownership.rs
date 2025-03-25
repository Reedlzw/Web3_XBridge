use {
    crate::{
        bridge_in::ContractConfig,
        common::XBridgeErrorCode,
    },
    anchor_lang::prelude::*,
};


#[derive(Accounts)]
pub struct TransferOwnershipContext<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut, 
        has_one = owner,
        seeds = [b"contract_config"],
        bump
    )]
    pub contract_config: Account<'info, ContractConfig>,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy)]
pub struct TransferOwnershipParams {
    new_owner: Pubkey
}

pub fn transfer_ownership(
    ctx: Context<TransferOwnershipContext>,
    params: TransferOwnershipParams,
) -> Result<()> {
    let contract_config = &mut ctx.accounts.contract_config;

    require!(
        params.new_owner != Pubkey::default() &&
        params.new_owner != contract_config.owner &&
        params.new_owner != contract_config.pending_owner,
        XBridgeErrorCode::InvalidPendingOwner
    );
    contract_config.pending_owner = params.new_owner;

    msg!(
        "OwnershipTransferStarted: previous_owner: {}, new_owner: {}",
        contract_config.owner,
        contract_config.pending_owner
    );
    Ok(())
}