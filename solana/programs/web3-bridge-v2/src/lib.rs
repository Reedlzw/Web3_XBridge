// Copyright (c) 2023-2025 OKX.com
// Licensed under the MIT License

pub mod bridge_out;
pub mod bridge_in;
pub mod common;

use {
    anchor_lang::prelude::*, 
    bridge_out::*, 
    bridge_in::*,
};

declare_id!("okxBd18urPbBi2vsExxUDArzQNcju2DugV9Mt46BxYE");

#[program]
pub mod web3_bridge_v2 {
    use super::*;

    // bridge_in
    pub fn initialize(ctx: Context<InitializeContext>, data: InitializeParams) -> Result<()> {
        bridge_in::initialize(ctx, data)
    }

    pub fn transfer_ownership(
        ctx: Context<TransferOwnershipContext>,
        params: TransferOwnershipParams,
    ) -> Result<()> {
        bridge_in::transfer_ownership(ctx, params)
    }

    pub fn accept_ownership(
        ctx: Context<AcceptOwnershipContext>,
    ) -> Result<()> {
        bridge_in::accept_ownership(ctx)
    }

    pub fn set_mpc(ctx: Context<SetMpcContext>, data: SetMpcParams) -> Result<()> {
        bridge_in::set_mpc(ctx, data)
    }

    pub fn set_oracle(ctx: Context<SetOracleContext>, data: SetOracleParams) -> Result<()> {
        bridge_in::set_oracle(ctx, data)
    }

    pub fn pause(ctx: Context<PauseContext>) -> Result<()> {
        bridge_in::pause(ctx)
    }
    
    pub fn unpause(ctx: Context<UnPauseContext>) -> Result<()> {
        bridge_in::unpause(ctx)
    }

    pub fn verify<'info>(
        _ctx: Context<'_, '_, '_, 'info, Verify<'info>>,
        data: VerifyArgs,
    ) -> Result<()> {
        bridge_in::verify(_ctx, data)
    }
    
    pub fn claim<'info>(
        _ctx: Context<'_, '_, '_, 'info, Claim<'info>>, 
        data: ClaimArgs,
    ) -> Result<()> {
        bridge_in::claim(_ctx, data)
    }

    pub fn claim_to_sol<'info>(
        _ctx: Context<'_, '_, '_, 'info, Claim<'info>>, 
        data: ClaimArgs,
    ) -> Result<()> {
        bridge_in::claim_to_sol(_ctx, data)
    }

    pub fn refund<'info>(
        _ctx: Context<'_, '_, '_, 'info, Refund<'info>>, 
        data: RefundArgs, 
    ) -> Result<()> {
        bridge_in::refund(_ctx, data)
    }

    // bridge_out
    pub fn bridge_to_log<'info>(
        ctx: Context<'_, '_, '_, 'info, BridgeTo<'info>>,
        data: BridgeToArgs,
    ) -> Result<()> {
        bridge_out::bridge_to_log(ctx, data)
    }

    // bridge_out + spl commission  cctp、wormhole
    pub fn bridge_to_log_splcommission<'info>(
        ctx: Context<'_, '_, '_, 'info, BridgeToSplCommission<'info>>,
        data: BridgeToCommissionArgs,
    ) -> Result<()> {
        bridge_out::bridge_to_log_splcommission(ctx, data)
    }

    // bridge_out + sol commission  wormhole
    pub fn bridge_to_log_solcommission<'info>(
        ctx: Context<'_, '_, '_, 'info, BridgeToSolCommission<'info>>,
        data: BridgeToCommissionArgs,
    ) -> Result<()> {
        bridge_out::bridge_to_log_solcommission(ctx, data)
    }

}