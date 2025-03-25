use {
    crate::{
        bridge_in::ContractConfig,
        common::{
            XBridgeErrorCode,
            DEPLOYER_KEY_STR,
        },
    },
    anchor_lang::prelude::*,
};


#[derive(Accounts)]
pub struct InitializeContext<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account()]
    pub contract_controller: Signer<'info>,

    #[account()]
    /// CHECK: MPC address
    pub mpc: AccountInfo<'info>,

    // contractConfig state account
    #[account(
        init,
        payer = payer,
        space = 8 + ContractConfig::INIT_SPACE,
        seeds = [b"contract_config"],
        bump
    )]
    pub contract_config:  Account<'info, ContractConfig>,

    /// System Program.
    /// "11111111111111111111111111111111"
    /// CHECK: fixed
    pub system_program: Program<'info, System>,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy)]
pub struct InitializeParams {
    pub oracle: [u8; 20],
}


pub fn initialize(ctx: Context<InitializeContext>, data: InitializeParams) -> Result<()> {
    let contract_config = &mut ctx.accounts.contract_config;

    require!(
        ctx.accounts.contract_controller.key().to_string() == DEPLOYER_KEY_STR,
        XBridgeErrorCode::Unauthorized
    );

    contract_config.owner = ctx.accounts.contract_controller.key();
    contract_config.pending_owner = Pubkey::default();
    contract_config.paused = false;
    contract_config.oracle = data.oracle;
    contract_config.mpc = ctx.accounts.mpc.key();

    msg!(
        "Contract initialized by controller: {}. Owner: {}, Pending Owner: {}, Oracle: {:?}, MPC: {}, Paused: {}",
        ctx.accounts.contract_controller.key(),
        contract_config.owner,
        contract_config.pending_owner,
        hex::encode(contract_config.oracle),
        contract_config.mpc,
        contract_config.paused
    );
    Ok(())
}