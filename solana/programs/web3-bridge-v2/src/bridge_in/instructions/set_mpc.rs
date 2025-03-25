use {
    crate::bridge_in::ContractConfig,
    anchor_lang::prelude::*,
};


#[derive(Accounts)]
pub struct SetMpcContext<'info> {
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
pub struct SetMpcParams {
    pub new_mpc: Pubkey,
}

pub fn set_mpc(ctx: Context<SetMpcContext>, data: SetMpcParams) -> Result<()> {
    let contract_config = &mut ctx.accounts.contract_config;

    contract_config.mpc = data.new_mpc;

    msg!(
        "MPC address updated to: {} by owner: {}",
        data.new_mpc,
        ctx.accounts.owner.key()
    );
    Ok(())
}