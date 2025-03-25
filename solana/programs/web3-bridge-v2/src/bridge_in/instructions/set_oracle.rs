use {
    crate::bridge_in::ContractConfig,
    anchor_lang::prelude::*,
};


#[derive(Accounts)]
pub struct SetOracleContext<'info> {
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
pub struct SetOracleParams {
    pub new_oracle: [u8; 20],
}

pub fn set_oracle(ctx: Context<SetOracleContext>, data: SetOracleParams) -> Result<()> {
    let contract_config = &mut ctx.accounts.contract_config;

    contract_config.oracle = data.new_oracle;

    msg!(
        "Oracle address updated to: {:?} by owner: {}",
        hex::encode(data.new_oracle),
        ctx.accounts.owner.key()
    );
    Ok(())
}