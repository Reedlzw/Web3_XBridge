use {
    crate::{
        bridge_in::ContractConfig,
        common::XBridgeErrorCode,
    },
    anchor_lang::prelude::*,
};


#[derive(Accounts)]
pub struct UnPauseContext<'info> {
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

pub fn unpause(ctx: Context<UnPauseContext>) -> Result<()> {
    let contract_config = &mut ctx.accounts.contract_config;

    require!(
        contract_config.paused,
        XBridgeErrorCode::NotPaused
    );
    contract_config.paused = false;

    msg!(
        "Contract unpaused by owner: {}",
        ctx.accounts.owner.key()
    );
    Ok(())
}