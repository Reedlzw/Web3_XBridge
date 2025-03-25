use {
    crate::{
        bridge_out::{
            BridgeResult, BridgeTo, BridgeToArgs, SwapType
        },
        common::{
            wormhole_core_program as Wormhole, wormhole_token_bridge_program as TokenBridge, wrapped_sol
        }
    },
    anchor_lang::{
        prelude::*, 
        solana_program::{
            instruction::Instruction, program::invoke_signed, system_program, sysvar
        },
    },
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{spl_token, Mint, TokenAccount}
    },
};


pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, BridgeTo<'info>>, data: BridgeToArgs, bridge_to_wormhole_args: BridgeToWromholeArgs) -> Result<BridgeResult> {

    // msg!("mint address: {}", ctx.accounts.mint.to_account_info().key);
    let bridge_to_wormhole = BridgeToWormhole {
        // account in bridge_to.accounts
        payer: ctx.accounts.payer.clone(),
        mint: ctx.accounts.mint.clone(),
        user_token_account: ctx.accounts.user_token_account.clone(),
        associated_token_program: ctx.accounts.associated_token_program.clone(),
        token_program: ctx.accounts.token_program.clone(),
        system_program: ctx.accounts.system_program.clone(),
        // account in bridge_to.remaining_accounts
        config: ctx.remaining_accounts[0].to_account_info(),
        custody_token: ctx.remaining_accounts[1].to_account_info(),
        transfer_authority: ctx.remaining_accounts[2].to_account_info(),
        custody_authority: ctx.remaining_accounts[3].to_account_info(),
        core_bridge_config: ctx.remaining_accounts[4].to_account_info(),
        core_message: ctx.remaining_accounts[5].to_account_info(),
        core_emitter: ctx.remaining_accounts[6].to_account_info(),
        core_emitter_sequence: ctx.remaining_accounts[7].to_account_info(),
        core_fee_collector: ctx.remaining_accounts[8].to_account_info().into(),
        clock: ctx.remaining_accounts[9].to_account_info(),
        rent: ctx.remaining_accounts[10].to_account_info(),
        core_bridge_program: ctx.remaining_accounts[11].to_account_info(),
        token_bridge_program: ctx.remaining_accounts[12].to_account_info(),
    };

    // 01. prepare token
    match data.swap_type {
        // a.if BRIDGE,, first transfer native-sol to user ATA
        SwapType::BRIDGE => {
            match ctx.accounts.mint.to_account_info().key {
                // native-sol
                key if key == &wrapped_sol::ID => { 
                    // transfer native-sol to user ATA
                    anchor_lang::system_program::transfer(
                        CpiContext::new(
                            bridge_to_wormhole.token_program.to_account_info(),
                            anchor_lang::system_program::Transfer {
                                from: bridge_to_wormhole.payer.to_account_info(),
                                to: bridge_to_wormhole.user_token_account.to_account_info(),
                            },
                        ),
                        data.amount
                    )?;
        
                    anchor_spl::token::sync_native(
                        CpiContext::new(
                            bridge_to_wormhole.token_program.to_account_info(),
                            anchor_spl::token::SyncNative {
                                account: bridge_to_wormhole.user_token_account.to_account_info(),
                            },
                        )
                    )?;
                }, 
                // normal-spl-token do nothing here
                _ => { 
                    // do nothing here
                }
            }
        },
        // b.if SWAPANDBRIDGE, will receive wrapped-sol
        SwapType::SWAPANDBRIDGE => {
            // do nothing here
        }
    }

    // 02.Delegate spending to Token Bridge program's authority signer.
    anchor_spl::token::approve(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Approve {
                to: bridge_to_wormhole.user_token_account.to_account_info(),
                delegate: bridge_to_wormhole.transfer_authority.to_account_info(),
                authority: bridge_to_wormhole.payer.to_account_info(),
            },
        ),
        data.amount,
    )?;

    // 03.invoke the wormhole program
    let mut account_metas = vec![
        AccountMeta::new(bridge_to_wormhole.payer.key(), true),
        AccountMeta::new_readonly(bridge_to_wormhole.config.key(), false),
        AccountMeta::new(bridge_to_wormhole.user_token_account.key(), false),
        AccountMeta::new(bridge_to_wormhole.mint.key(), false),
        AccountMeta::new(bridge_to_wormhole.custody_token.key(), false),
        AccountMeta::new_readonly(bridge_to_wormhole.transfer_authority.key(), false),
        AccountMeta::new_readonly(bridge_to_wormhole.custody_authority.key(), false),
        AccountMeta::new(bridge_to_wormhole.core_bridge_config.key(), false),
        AccountMeta::new(bridge_to_wormhole.core_message.key(), true),
        AccountMeta::new_readonly(bridge_to_wormhole.core_emitter.key(), false),
        AccountMeta::new(bridge_to_wormhole.core_emitter_sequence.key(), false),
        AccountMeta::new(bridge_to_wormhole.core_fee_collector.as_ref().map(|info| *info.key).unwrap_or(crate::ID), false),
        AccountMeta::new_readonly(bridge_to_wormhole.clock.key(), false),
        // AccountMeta::new(bridge_to_wormhole.payer.key(), true), // if use TransferTokensWithPayloadNative ,we need this
        // Dependencies
        AccountMeta::new_readonly(bridge_to_wormhole.rent.key(), false),
        AccountMeta::new_readonly(bridge_to_wormhole.system_program.key(), false),
        // Program
        AccountMeta::new_readonly(bridge_to_wormhole.core_bridge_program.key(), false),
        AccountMeta::new_readonly(bridge_to_wormhole.token_program.key(), false),
    ];

    let args_vec: Vec<u8>;
    if bridge_to_wormhole_args.redeemer == [0u8; 32] {
        let mut _args_vec = &mut vec![LegacyInstruction::TransferTokensNative as u8];
        let args = TransferTokensArgs {
            nonce: bridge_to_wormhole_args.nonce as u32,
            amount: data.amount,
            relayer_fee: 0,
            recipient: data.to.try_into().unwrap(),
            recipient_chain: data.to_chain_id as u16,
        };
        _args_vec.append(&mut AnchorSerialize::try_to_vec(&args).unwrap());
        args_vec = _args_vec.to_vec();
    } else {
        account_metas.insert(13, AccountMeta::new(bridge_to_wormhole.payer.key(), true));
        let mut _args_vec = &mut vec![LegacyInstruction::TransferTokensWithPayloadNative as u8];
        let args = TransferTokensWithPayloadArgs {
            nonce: bridge_to_wormhole_args.nonce as u32,
            amount: data.amount,
            redeemer: bridge_to_wormhole_args.redeemer,
            redeemer_chain: data.to_chain_id as u16,
            payload: data.to,
            cpi_program_id: None,
        };
        _args_vec.append(&mut AnchorSerialize::try_to_vec(&args).unwrap());
        args_vec = _args_vec.to_vec();
    }

    let ix = Instruction {
        program_id: TokenBridge::id(), 
        data: args_vec,
        accounts: account_metas,
    };

    // find core_message pubkey and bumps for create signers
    let nonce_binding = bridge_to_wormhole_args.nonce.to_le_bytes();
    let core_message_seeds:&[&[u8]] = &[
        b"bridged",
        &nonce_binding.as_ref()[..],
    ];

    let core_message = Pubkey::find_program_address(core_message_seeds, &crate::ID);

    let core_message_signer = &[
        core_message_seeds[0],
        core_message_seeds[1],
        &[core_message.1],
    ];

    let signer_seeds = [
            &core_message_signer[..]
    ];

    // execute instruction
    invoke_signed(
        &ix,
        &bridge_to_wormhole.to_account_infos(),
        &signer_seeds
    )?;

    Ok(BridgeResult{
        ext: "".to_string(),
    })

}

#[derive(Debug, AnchorDeserialize, AnchorSerialize, Clone)]
pub struct BridgeToWromholeArgs {
    pub nonce: u64,
    pub redeemer: [u8; 32],
    pub payload: Vec<u8>,
}

impl BridgeToWromholeArgs {
    pub fn try_from_vec(data: &[u8]) -> Result<BridgeToWromholeArgs> {
        let nonce = u64::from_be_bytes(<[u8; 8]>::try_from(&data[0..8]).unwrap());
        let redeemer = <[u8; 32]>::try_from(&data[8..40]).unwrap();
        let payload = data[40..].to_vec();
        Ok(BridgeToWromholeArgs{
            nonce,
            redeemer,
            payload,
        })
    }
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TransferTokensArgs {
    pub nonce: u32,
    pub amount: u64,
    pub relayer_fee: u64,
    pub recipient: [u8; 32],
    pub recipient_chain: u16,
}

impl TransferTokensArgs {
    pub fn try_from_vec(data: Vec<u8>) -> Result<TransferTokensArgs> {
        let nonce = u32::from_le_bytes(<[u8; 4]>::try_from(&data[0..4]).unwrap());
        let amount = u64::from_le_bytes(<[u8; 8]>::try_from(&data[4..12]).unwrap());
        let relayer_fee = u64::from_le_bytes(<[u8; 8]>::try_from(&data[12..20]).unwrap());
        let recipient = <[u8; 32]>::try_from(&data[20..52]).unwrap();
        let recipient_chain = u16::from_le_bytes(<[u8; 2]>::try_from(&data[52..54]).unwrap());
        Ok(TransferTokensArgs{
            nonce,
            amount,
            relayer_fee,
            recipient,
            recipient_chain,
        })
    }
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct TransferTokensWithPayloadArgs {
    pub nonce: u32,
    pub amount: u64,
    pub redeemer: [u8; 32],
    pub redeemer_chain: u16,
    pub payload: Vec<u8>,
    pub cpi_program_id: Option<Pubkey>,
}

#[derive(Accounts)]
// #[instruction(nonce: u64)]
#[instruction(bridge_to_wormhole_args: BridgeToWromholeArgs)]
pub struct BridgeToWormhole<'info> {
    #[account(mut)]
    /// Payer.
    /// This account is user to pay for the transaction and bridge token.
    /// CHECK: This account is user to pay for the transaction and bridge token.
    pub payer: Signer<'info>,

    #[account(mut)]
    /// CHECK: Mint (read-only).
    pub mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = payer,
    )]
    /// User Token Account
    /// ATA: mint = mint, owner = payer,
    /// CHECK: This Account is used to send token for user,
    pub user_token_account: Account<'info, TokenAccount>,

    /// Associated Token Program
    /// "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
    /// CHECK: fixed
    #[account(address = AssociatedToken::id())]
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// SPL Token Program
    /// "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    /// CHECK: fixed
    #[account(address = spl_token::id())]
    pub token_program: AccountInfo<'info>,

    /// System Program.
    /// "11111111111111111111111111111111"
    /// CHECK: fixed
    #[account(address = system_program::id())]
    pub system_program: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [
            b"config".as_ref()
        ],
        bump,
        seeds::program = token_bridge_program.key(),
    )]
    /// CHECK: Mint (read-only).
    pub config: AccountInfo<'info>,

    #[account( 
        mut, 
        seeds = [mint.key().as_ref()], 
        bump, 
        seeds::program = token_bridge_program.key(),
    )]
    /// CHECK: Custody Token Account (mut, seeds = \[mint.key\], seeds::program = token_bridge_program).
    pub custody_token: AccountInfo<'info>,
    
    #[account(
        seeds = [b"authority_signer".as_ref()],
        bump,
        seeds::program = token_bridge_program.key(),
    )]
    /// CHECK: Transfer Authority (read-only, seeds = \["authority_signer"\], seeds::program = token_bridge_program).
    pub transfer_authority: AccountInfo<'info>,
    
    #[account(
        seeds = [b"custody_signer".as_ref()],
        bump,
        seeds::program = token_bridge_program.key(),
    )]
    /// CHECK: Custody Authority (read-only, seeds = \["custody_signer"\], seeds::program = token_bridge_program).
    pub custody_authority: AccountInfo<'info>,
    
    #[account(
        mut,
        seeds = [b"Bridge".as_ref()],
        bump,
        seeds::program = core_bridge_program.key(),
    )]
    /// CHECK: Core Bridge Config (read-only, seeds = \["Bridge"\], seeds::program = core_bridge_program).
    pub core_bridge_config: AccountInfo<'info>,

    /// CHECK: Core Bridge Program.
    #[account(address = Wormhole::id())]
    pub core_bridge_program: AccountInfo<'info>,

    /// CHECK: Token Bridge Program
    #[account(address = TokenBridge::id())]
    pub token_bridge_program: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [ b"bridged".as_ref(), &bridge_to_wormhole_args.nonce.to_le_bytes() ],
        bump,
        seeds::program = crate::ID,
    )]
    /// CHECK: Core Bridge Message (mut) ( seeds = \["bridged"\, nonce], seeds::program = web3_bridge_v2).
    pub core_message: AccountInfo<'info>,
    
    #[account(
        seeds = [b"emitter".as_ref()],
        bump,
        seeds::program = token_bridge_program.key(),
    )]
    /// CHECK: Core Bridge Emitter (read-only, seeds = \["emitter"\], seeds::program = token_bridge_program).
    pub core_emitter: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [
            b"Sequence".as_ref(),
            core_emitter.key().as_ref()
        ],
        bump,
        seeds::program = core_bridge_program.key(),
    )]
    /// Core Bridge Emitter Sequence (mut).
    ///
    /// Seeds = \["Sequence", emitter.key\], seeds::program = core_bridge_program.
    ///
    /// CHECK: This account is used to determine the sequence of the Wormhole message for the provided emitter.
    pub core_emitter_sequence: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"fee_collector".as_ref()],
        bump,
        seeds::program = core_bridge_program.key(),
    )]
    /// Core Bridge Fee Collector (optional, read-only).
    ///
    /// Seeds = \["fee_collector"\], seeds::program = core_bridge_program.
    ///
    /// CHECK: This account is used to collect fees.
    pub core_fee_collector: Option<AccountInfo<'info>>,

    /// CHECK: Clock Sysvar.
    #[account(address = sysvar::clock::id())]
    pub clock: AccountInfo<'info>,

    /// CHECK: Rent Sysvar.
    #[account(address = sysvar::rent::id())]
    pub rent: AccountInfo<'info>,
}

/// NOTE: No more instructions should be added to this enum. Instead, add them as Anchor instruction
/// handlers, which will inevitably live in lib.rs.
#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone)]
pub enum LegacyInstruction {
    Initialize,
    AttestToken,
    CompleteTransferNative,
    CompleteTransferWrapped,
    TransferTokensWrapped,
    TransferTokensNative,
    RegisterChain,
    CreateOrUpdateWrapped,
    UpgradeContract,
    CompleteTransferWithPayloadNative,
    CompleteTransferWithPayloadWrapped,
    TransferTokensWithPayloadWrapped,
    TransferTokensWithPayloadNative,
}

#[event]
pub struct CrossChainDataEvent{
    adaptor_id: u8,
    swap_type: u8,
    amount: u64,
    encoded: String,
    initiator: String,
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn decode_data(){
        // let data = "".try_to_vec();
        // msg!("data: {:?}", data);
        let data: Vec<u8> = vec![108, 118, 2, 0, 48, 57, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2];
        let res = TransferTokensArgs::try_from_slice(&data);
        msg!("res: {:?}", res.unwrap());
    }

    #[test]
    fn encode_data(){
        
        let data = TransferTokensArgs{
            nonce: 0x2766c,
            amount: 0x3930,
            relayer_fee: 0,
            recipient: [0; 32],
            recipient_chain: 0x200,
        };
        msg!("data: {:?}", data);
        // msg!("data: {:?}", "0x".to_string() + &vec_to_hex_string(data.recipient.to_vec()),);
        let res = AnchorSerialize::try_to_vec(&data);
        msg!("data: {:?}", res.unwrap());
    }

    #[test]
    fn encode_wormhoe_args() {
        let data = BridgeToWromholeArgs{
            nonce: 0x2766c,
            redeemer: [0; 32],
            payload: vec![],
        };

        msg!("data: {:?}", data);
        msg!("data: {:?}", AnchorSerialize::try_to_vec(&data).unwrap());

        let data2: Vec<u8> = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x76, 0x6c, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x35, 0x85, 0x06, 0xb4, 0xc5, 0xc4, 0x41, 0x87, 0x3a, 0xde, 0x42, 0x9c, 0x5a, 0x2b, 0xe7, 0x77, 0x57, 0x8e, 0x2c, 0x6f];
        msg!("res: {:?}", BridgeToWromholeArgs::try_from_vec(&data2).unwrap());
        // msg!("res: {:?}", vec_to_hex_string(BridgeToWromholeArgs::try_from_vec(&data2).unwrap().redeemer.to_vec()));
    }

}