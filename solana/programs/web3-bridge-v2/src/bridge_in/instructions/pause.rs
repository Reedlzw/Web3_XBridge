use {
    crate::{
        bridge_in::ContractConfig,
        common::XBridgeErrorCode,
    },
    anchor_lang::prelude::*,
};


#[derive(Accounts)]
pub struct PauseContext<'info> {
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

pub fn pause(ctx: Context<PauseContext>) -> Result<()> {
    let contract_config = &mut ctx.accounts.contract_config;

    require!(
        !contract_config.paused,
        XBridgeErrorCode::AlreadyPaused
    );
    contract_config.paused = true;

    msg!(
        "Contract paused by owner: {}",
        ctx.accounts.owner.key()
    );
    Ok(())
}