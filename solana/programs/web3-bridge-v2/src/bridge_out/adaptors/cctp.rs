use {
    crate::{
        bridge_out::{
            BridgeTo,
            BridgeToArgs,
            RelayerFee,
            BridgeResult,
        },
        common::{
            cctp_message_program as CCTPMessage, 
            cctp_program as CCTP, 
        }
    },
    anchor_lang::{
        prelude::*, 
        solana_program::{
            system_program,
            instruction::Instruction, 
            program::invoke
        }
    },
    anchor_spl::token::{
        Token, 
        Mint, 
        TokenAccount, 
        transfer, 
        Transfer
    },
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BridgeToCctpArgs {
    pub amount: u64,
    pub destination_domain: u32,
    pub mint_recipient: Pubkey,
}

impl BridgeToCctpArgs {
    pub fn try_from_vec(data: &[u8]) -> Result<BridgeToCctpArgs> {
        let decoded_args = BridgeToCctpArgs::try_from_slice(&data)?;
        Ok(decoded_args)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RedeemToCctpArgs {
    pub amount: u64,
}

impl RedeemToCctpArgs {
    pub fn try_from_vec(data: &[u8]) -> Result<RedeemToCctpArgs> {
        let decoded_args = RedeemToCctpArgs::try_from_slice(&data)?;
        Ok(decoded_args)
    }
}

#[derive(Accounts)]
#[instruction(bridge_to_cctp_args: BridgeToCctpArgs)]
pub struct BridgeToCctp<'info> {
    #[account(mut)]
    /// Payer.
    /// This account is user to pay for the transaction and bridge token.
    /// CHECK: This account is user to pay for the transaction and bridge token.
    pub payer: Signer<'info>,

    #[account(
        seeds = [b"sender_authority".as_ref()],
        bump,
        seeds::program = token_messenger_minter_program.key(),
    )]
    /// Token Message Account
    /// PDA: seeds = [b"sender_authority"], seeds::program = "cctp_program".
    /// CHECK: This account is a PDA account, which is used for cctp.
    pub sender_authority_pda: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = payer,
    )]
    /// User Token Account
    /// ATA: mint = mint, owner = payer,
    /// CHECK: This Account is used to send token for user,
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut,
        seeds = [b"message_transmitter".as_ref()],
        bump,
        seeds::program = message_transmitter_program.key(),
    )]
    /// Message Transmitter Account
    /// PDA: seeds = [b"message_transmitter"], seeds::program = "cctp_message_program".
    /// CHECK: This account is a PDA account, which is used for cctp.
    pub message_transmitter: AccountInfo<'info>,

    #[account(
        seeds = [b"token_messenger".as_ref()],
        bump,
        seeds::program = token_messenger_minter_program.key(),
    )]
    /// Token Message Account
    /// PDA: seeds = [b"token_messenger"], seeds::program = "cctp_program".
    /// CHECK: This account is a PDA account, which is used for cctp.
    pub token_messenger:  AccountInfo<'info>,

    #[account(
        seeds = [b"remote_token_messenger".as_ref(),bridge_to_cctp_args.destination_domain.to_string().as_bytes()],
        bump,
        seeds::program = token_messenger_minter_program.key(),
    )]
    /// Remote Token Account
    /// PDA: seeds = [b"remote_token_messenger",dest_domain.to_string()], seeds::program = "cctp_program".
    /// CHECK: This account is a PDA account, which is used for cctp.
    pub remote_token_messenger: AccountInfo<'info>,

    #[account(
        seeds = [b"token_minter".as_ref()],
        bump,
        seeds::program = token_messenger_minter_program.key(),
    )]
    /// Token Minter Account
    /// PDA: seeds = [b"token_minter"], seeds::program = "cctp_program".
    /// CHECK: This account is a PDA account, which is used for cctp.
    pub token_minter: AccountInfo<'info>,
 
    #[account(
        mut,
        seeds = [b"local_token".as_ref(), mint.key().as_ref()],
        bump,
        seeds::program = token_messenger_minter_program.key(),
    )]
    /// Local Token Account
    /// PDA: seeds = [b"local_token",mint.key()], seeds::program = "cctp_program".
    /// CHECK: This account is a PDA account, which is used for cctp.
    pub local_token: AccountInfo<'info>,

    #[account(mut)]
    /// tokenMint
    /// give token mint
    /// CHECK:
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    /// CHECK: Account to store MessageSent event data in. Any non-PDA uninitialized address.
    pub message_sent_event_data: AccountInfo<'info>,

    /// message_transmitter Program ID
    /// "CCTPmbSD7gX1bxKPAmg77w8oFzNFpaQiQUWD43TKaecd"
    /// CHECK: fixed
    #[account(address = CCTPMessage::id())]
    pub message_transmitter_program: AccountInfo<'info>,

    /// cctp_token_messenger_minter Program ID
    /// "CCTPiPYPc6AsJuwueEnWgSgucamXDZwBd53dQ11YiKX3"
    /// CHECK: fixed
    #[account(address = CCTP::id())]
    pub token_messenger_minter_program: AccountInfo<'info>,

    /// SPL Token Program
    /// "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    /// CHECK: fixed
    #[account(address = Token::id())]
    pub token_program: AccountInfo<'info>,

    /// System Program.
    /// "11111111111111111111111111111111"
    /// CHECK: fixed
    #[account(address = system_program::id())]
    pub system_program: AccountInfo<'info>,
    
    #[account(
            seeds = [b"__event_authority".as_ref()],
            bump,
            seeds::program = token_messenger_minter_program.key(),
    )]
    /// Event Authority Account
    /// PDA: seeds = [b"__event_authority"], seeds::program = "cctp_program".
    /// CHECK: This account is a PDA account, which is used for cctp.
    pub event_authority: AccountInfo<'info>,

}

#[event]
pub struct CrossChainDataEvent{
    adaptor_id: u8,
    swap_type: u8,
    amount: u64,
    destination_domain: u32,
    mint_recipient: Pubkey,
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CrossChainData{
    adaptor_id: u8,
    swap_type: u8,
    amount: u64,
    destination_domain: u32,
    mint_recipient: Pubkey,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, BridgeTo<'info>>, mut data: BridgeToArgs ) -> Result<BridgeResult> {

    // redeem fee
    let redeem_fee_wrap = RedeemToCctpArgs::try_from_vec(&data.data);
    if redeem_fee_wrap.is_ok() {
        let redeem_fee = redeem_fee_wrap.unwrap();
        let cpi_context = CpiContext::new(
            ctx.accounts.mint.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_account.to_account_info(),
                to: ctx.remaining_accounts[10].to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            }
        );
        transfer(cpi_context, redeem_fee.amount)?;    
        data.amount -= redeem_fee.amount;

        let relayer_fee: RelayerFee = RelayerFee {
            amount: redeem_fee.amount,
            mint: ctx.accounts.mint.key().to_string(),
            to: ctx.remaining_accounts[10].key().to_string(),
        };
        msg!("RelayerFee:{}", serde_json::to_string(&relayer_fee).unwrap());
    } else {
        // compatible with current version with no redeem fee
    }
    
    // bridge_to_cctp
    let mut new_data = vec![215u8, 60u8, 61u8, 46u8, 114u8, 55u8, 128u8, 176u8];
    new_data.extend_from_slice(&data.amount.to_le_bytes());
    new_data.extend_from_slice(&(data.to_chain_id as u32).to_le_bytes());
    new_data.extend_from_slice(&data.to);

    let bridge_to_cctp = BridgeToCctp{
        // account in bridge_to.accounts
        payer: ctx.accounts.payer.clone(),
        mint: ctx.accounts.mint.clone(),
        user_token_account: ctx.accounts.user_token_account.clone(),
        token_program: ctx.accounts.token_program.clone(),
        system_program: ctx.accounts.system_program.clone(),
        // account in bridge_to.remaining_accounts
        sender_authority_pda: ctx.remaining_accounts[0].to_account_info(),
        message_transmitter: ctx.remaining_accounts[1].to_account_info(),
        token_messenger: ctx.remaining_accounts[2].to_account_info(),
        remote_token_messenger:ctx.remaining_accounts[3].to_account_info(),
        token_minter:ctx.remaining_accounts[4].to_account_info(),
        local_token:ctx.remaining_accounts[5].to_account_info(),
        message_sent_event_data: ctx.remaining_accounts[6].to_account_info(),
        message_transmitter_program: ctx.remaining_accounts[7].to_account_info(),
        token_messenger_minter_program: ctx.remaining_accounts[8].to_account_info(),
        event_authority: ctx.remaining_accounts[9].to_account_info(),
    };

    let ix = Instruction {
        program_id: CCTP::id(),
        data: new_data,
        accounts:vec![
            AccountMeta::new(bridge_to_cctp.payer.key(), true),
            AccountMeta::new(bridge_to_cctp.payer.key(), true),
            AccountMeta::new_readonly(bridge_to_cctp.sender_authority_pda.key(), false),
            AccountMeta::new(bridge_to_cctp.user_token_account.key(), false),
            AccountMeta::new(bridge_to_cctp.message_transmitter.key(), false),
            AccountMeta::new_readonly(bridge_to_cctp.token_messenger.key(), false),
            AccountMeta::new_readonly(bridge_to_cctp.remote_token_messenger.key(), false),
            AccountMeta::new_readonly(bridge_to_cctp.token_minter.key(), false),
            AccountMeta::new(bridge_to_cctp.local_token.key(), false),
            AccountMeta::new(bridge_to_cctp.mint.key(), false),
            AccountMeta::new(bridge_to_cctp.message_sent_event_data.key(), true),
            AccountMeta::new_readonly(bridge_to_cctp.message_transmitter_program.key(), false),
            AccountMeta::new_readonly(bridge_to_cctp.token_messenger_minter_program.key(), false),
            AccountMeta::new_readonly(bridge_to_cctp.token_program.key(), false),
            AccountMeta::new_readonly(bridge_to_cctp.system_program.key(), false),
            AccountMeta::new_readonly(bridge_to_cctp.event_authority.key(), false),
            AccountMeta::new_readonly(bridge_to_cctp.token_messenger_minter_program.key(), false),
        ],
    };

    msg!("Invoke the cctp program with the following data: {:?}");

    invoke(
        &ix,
        &bridge_to_cctp.to_account_infos()
    )?;

    Ok(BridgeResult{
        ext: "".to_string(),
    })

}


#[cfg(test)]
mod test {
    use super::*;
    use anchor_lang::solana_program::hash as hash;
    pub const SIGHASH_GLOBAL_NAMESPACE: &str = "global";
    
    //[215, 60, 61, 46, 114, 55, 128, 176]
    // d7 3c 3d 2e 72 37 80 b0
    #[test]
    pub fn sighash() {
        let name: &str = "deposit_for_burn";
        let preimage = format!("{}:{}", SIGHASH_GLOBAL_NAMESPACE, name);
    
        let mut sighash = [0u8; 8];
        sighash.copy_from_slice(&hash::hash(preimage.as_bytes()).to_bytes()[..8]);
        msg!("{:?}", sighash);
    }

    #[test]
    fn decode() {
        let hex_string = "a08601000000000001000000000000000000000000000000c380bedadc4935dd96ddc67dc057bed07738e8af";
        // let hex_string = "8050de190000000006000000000000000000000000000000694200f3298d80e966d7230b1cebe4fd264491a3";
        // match hex::decode(hex_string) {
        //     Ok(data) => {
        //         match BridgeToCctpArgs::try_from_slice(&data) {
        //             Ok(decoded_args) => println!("Decoded successfully: {:?}", decoded_args),
        //             Err(e) => println!("Failed to deserialize: {:?}", e),
        //         }
        //     },
        //     Err(e) => println!("Failed to decode hex string: {}", e),
        // }
        // let decoded_args = BridgeToCctpArgs::try_from_slice(&hex::decode(hex_string).unwrap()).unwrap();
        let res = RedeemToCctpArgs::try_from_vec(&hex::decode(hex_string).unwrap());
        if res.is_ok() {
            msg!("Decoded successfully: {:?}", res.unwrap());
        } else {
            msg!("Failed to deserialize: {:?}", res.unwrap_err());
        }
        // let mut new_data2 =  vec![215u8, 60u8, 61u8, 46u8, 114u8, 55u8, 128u8, 176u8];
        // let amount = 100000u64;
        // let destination_domain = 1u64 as u32;
        // let mint_recipient = vec![215u8, 60u8, 61u8, 46u8, 114u8, 55u8, 128u8, 176u8];
        
        // let mut new_data =  vec![215u8, 60u8, 61u8, 46u8, 114u8, 55u8, 128u8, 176u8];
        // new_data.extend_from_slice(&decoded_args.amount.to_le_bytes());
        // new_data.extend_from_slice(&decoded_args.destination_domain.to_le_bytes());
        // new_data.extend_from_slice(&decoded_args.mint_recipient.to_bytes());
        // new_data.extend_from_slice(&mint_recipient);


        // msg!("new_data: {:?}", new_data);


        // new_data2.extend_from_slice(&amount.to_le_bytes());
        // new_data2.extend_from_slice(&destination_domain.to_le_bytes());
        // new_data2.extend_from_slice(&mint_recipient);

        // msg!("new_data2: {:?}", new_data2);
    }

}