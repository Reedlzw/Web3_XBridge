use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace, Debug)]
pub struct ToSwapMessageState {
    pub is_used: bool, // 1
    pub authority: Pubkey, // 32                
    pub authority_program: Pubkey, // 32 
    pub data: [u8; 160],    // 5 * 32 = 160
}

#[account]
#[derive(InitSpace)]
pub struct ContractConfig {
    pub owner: Pubkey,          // The public key of the current program owner
    pub pending_owner: Pubkey,  // The public key of the pending program owner
    pub paused: bool,           // Boolean value indicating whether the program is paused
    pub oracle: [u8; 20],       // The oracle address
    pub mpc: Pubkey,            // The MPC (Multi-Party Computation) address
}