use {
    crate::bridge_in::ContractConfig,
    anchor_lang::prelude::*,
};


#[derive(Accounts)]
pub struct AcceptOwnershipContext<'info> {
    #[account(mut)]
    pub pending_owner: Signer<'info>,

    #[account(
        mut, 
        has_one = pending_owner,
        seeds = [b"contract_config"],
        bump
    )]
    pub contract_config: Account<'info, ContractConfig>,
}

pub fn accept_ownership(
    ctx: Context<AcceptOwnershipContext>,
) -> Result<()> {
    let contract_config = &mut ctx.accounts.contract_config;

    let previous_owner = contract_config.owner;
    contract_config.owner = contract_config.pending_owner;
    contract_config.pending_owner = Pubkey::default();

    msg!(
        "OwnershipTransferred: previous_owner: {}, new_owner: {}",
        previous_owner,
        contract_config.owner
    );
    Ok(())
}