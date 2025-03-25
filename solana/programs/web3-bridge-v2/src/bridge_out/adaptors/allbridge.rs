use {
    crate::{
        bridge_out::{BridgeResult, BridgeTo, BridgeToArgs},
        common::{
            allbridge_gas_program as GasProgram, allbridge_messager_program as MessagerProgram,
            allbridge_program as AllBridgeProgram, AllBridgeErrorCode,
        },
    },
    anchor_lang::{
        prelude::*,
        solana_program::{instruction::Instruction, program::invoke},
    },
    anchor_spl::token::{Mint, Token, TokenAccount},
};

pub const CHAIN_ID: u8 = 4;

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, BridgeTo<'info>>,
    data: BridgeToArgs,
    bridge_to_allbridge_args: BridgeToAllbridgeArgs,
) -> Result<BridgeResult> {

    let mut new_data = vec![204u8, 63u8, 169u8, 171u8, 186u8, 125u8, 86u8, 159u8];
    new_data.extend_from_slice(&bridge_to_allbridge_args.nonce);
    new_data.extend_from_slice(&data.to);
    new_data.extend_from_slice(&(data.to_chain_id as u8).to_le_bytes());
    new_data.extend_from_slice(&bridge_to_allbridge_args.receive_token);
    new_data.extend_from_slice(&(bridge_to_allbridge_args.vusd_amount as u64).to_le_bytes());

    require!(
        new_data.len() == 113,
        AllBridgeErrorCode::DataError
    );

    let bridge_to_allbridge = BridgeToAllbridge {
        // account in bridge_to.accounts
        payer: ctx.accounts.payer.clone(),
        mint: ctx.accounts.mint.clone(),
        user_token: ctx.accounts.user_token_account.clone(),
        token_program: ctx.accounts.token_program.clone(),
        system_program: ctx.accounts.system_program.clone(),
        //account in bridge_to.remaining_accounts
        lock: ctx.remaining_accounts[0].to_account_info(),
        config: ctx.remaining_accounts[1].to_account_info(),
        other_bridge_token: ctx.remaining_accounts[2].to_account_info(),
        messenger: ctx.remaining_accounts[3].to_account_info(),
        messenger_config: ctx.remaining_accounts[4].to_account_info(),
        sent_message_account: ctx.remaining_accounts[5].to_account_info(),
        messenger_gas_usage: ctx.remaining_accounts[6].to_account_info(),
        pool: ctx.remaining_accounts[7].to_account_info(),
        bridge_token: ctx.remaining_accounts[8].to_account_info(),
        gas_price: ctx.remaining_accounts[9].to_account_info(),
        this_gas_price: ctx.remaining_accounts[10].to_account_info(),
        chain_bridge: ctx.remaining_accounts[11].to_account_info(),
        bridge_authority: ctx.remaining_accounts[12].to_account_info(),
        allbridge_program: ctx.remaining_accounts[13].to_account_info(),
    };

    let ix = Instruction {
        program_id: AllBridgeProgram::id(),
        data: new_data,
        accounts: vec![
            AccountMeta::new(bridge_to_allbridge.payer.key(), true),
            AccountMeta::new(bridge_to_allbridge.payer.key(), true),
            AccountMeta::new(bridge_to_allbridge.lock.key(), false),
            AccountMeta::new_readonly(bridge_to_allbridge.mint.key(), false),
            AccountMeta::new(bridge_to_allbridge.config.key(), false),
            AccountMeta::new(bridge_to_allbridge.other_bridge_token.key(), false),
            AccountMeta::new_readonly(bridge_to_allbridge.messenger.key(), false),
            AccountMeta::new(bridge_to_allbridge.messenger_config.key(), false),
            AccountMeta::new(bridge_to_allbridge.sent_message_account.key(), false),
            AccountMeta::new_readonly(bridge_to_allbridge.messenger_gas_usage.key(), false),
            AccountMeta::new(bridge_to_allbridge.pool.key(), false),
            AccountMeta::new(bridge_to_allbridge.bridge_token.key(), false),
            AccountMeta::new_readonly(bridge_to_allbridge.gas_price.key(), false),
            AccountMeta::new_readonly(bridge_to_allbridge.this_gas_price.key(), false),
            AccountMeta::new_readonly(bridge_to_allbridge.chain_bridge.key(), false),
            AccountMeta::new(bridge_to_allbridge.user_token.key(), false),
            AccountMeta::new_readonly(bridge_to_allbridge.bridge_authority.key(), false),
            AccountMeta::new_readonly(bridge_to_allbridge.token_program.key(), false),
            AccountMeta::new_readonly(bridge_to_allbridge.system_program.key(), false),
        ],
    };
    invoke(
        &ix, 
        &bridge_to_allbridge.to_account_infos()
    )?;

    Ok(BridgeResult {
        ext: "".to_string(),
    })
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BridgeToAllbridgeArgs {
    pub nonce: [u8; 32],
    pub receive_token: [u8; 32],
    pub message_with_signer: [u8; 32],
    pub vusd_amount: u64,
}

impl BridgeToAllbridgeArgs {
    pub fn try_from_vec(data: &[u8]) -> Result<BridgeToAllbridgeArgs> {
        let decoded_args: BridgeToAllbridgeArgs = BridgeToAllbridgeArgs::try_from_slice(&data)?;
        Ok(decoded_args)
    }
}

#[derive(Accounts)]
#[instruction(bridge_to_allbridge_args: BridgeToAllbridgeArgs, bridge_to_args: BridgeToArgs)]
pub struct BridgeToAllbridge<'info> {
    #[account(mut)]
    /// Payer.
    /// This account is used to pay for the transaction and bridge token.
    /// CHECK: This account is used to pay for the transaction and bridge token.
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"lock", bridge_to_allbridge_args.nonce.as_ref()],
        bump,
        seeds::program = AllBridgeProgram::id(),
    )]
    /// Lock.
    /// PDA: seeds = [b"lock"], seeds::program = "allbridge_program"
    /// CHECK: This Account is used to create ATA account to locking tokens.
    pub lock: AccountInfo<'info>,

    #[account()]
    /// mint
    /// send token mint
    /// CHECK: Mint (read-only)
    pub mint: Account<'info, Mint>,

    #[account(mut,
        seeds = [b"config"],
        bump,
        seeds::program = AllBridgeProgram::id(),)]
    /// Config.
    /// PDA: seeds = [b"config"], seeds::program = "allbridge_program"
    /// CHECK: This Account is used to create ATA account to configuration data.
    pub config: AccountInfo<'info>,

    #[account(mut,
        seeds = [b"other_bridge_token", &[bridge_to_args.to_chain_id.try_into().unwrap()],bridge_to_allbridge_args.receive_token.as_ref()],
        bump,
        seeds::program = AllBridgeProgram::id(),)]
    /// Other Bridge Token.
    /// PDA: seeds = [b"other_bridge_token", destination_chain_id, receive_token], seeds::program = "allbridge_program"
    /// CHECK: This Account is used to create ATA account to hold the other bridge token
    pub other_bridge_token: AccountInfo<'info>,

    /// allbridge Messenger Program ID
    /// "AMsgYtqR3EXKfsz6Rj2cKnrYGwooaSk7BQGeyVBB5yjS"
    /// CHECK: fixed
    #[account(address = MessagerProgram::id())]
    pub messenger: AccountInfo<'info>,

    #[account(mut,
        seeds = [b"config"],
        bump,
        seeds::program = messenger.key(),)]
    /// Messenger Config.
    /// PDA: seeds = [b"config"], seeds::program = "messenger"
    /// CHECK: This account holds the messenger configuration data.
    pub messenger_config: AccountInfo<'info>,

    #[account(mut,
        seeds = [b"sent_message",&bridge_to_allbridge_args.message_with_signer],
        bump,
        seeds::program = messenger.key(),)]
    /// Sent Message Account.
    /// This account holds the sent message data.
    /// CHECK: This account holds the sent message data.
    pub sent_message_account: AccountInfo<'info>,

    #[account(
        seeds = [b"gas_usage",&[bridge_to_args.to_chain_id.try_into().unwrap()]],
        bump,
        seeds::program = messenger.key(),)]
    /// Messenger Gas Usage.
    /// PDA: seeds = [b"gas_usage", destination_chain_id], seeds::program = "messenger"
    /// CHECK: This account tracks the gas usage of the messenger.
    pub messenger_gas_usage: AccountInfo<'info>,

    #[account(mut)]
    /// Pool.
    /// CHECK: This account is used for the token pool.
    pub pool: AccountInfo<'info>,

    #[account(mut,
        seeds = [b"token", mint.to_account_info().key.as_ref()],
        bump,
        seeds::program = AllBridgeProgram::id(),)]
    /// Bridge Token.
    /// PDA: seeds = [b"token"], seeds::program = "allbridge_program"
    /// CHECK: This account holds the bridge token.
    pub bridge_token: AccountInfo<'info>,

    #[account(
        seeds = [b"price_v2",&[bridge_to_args.to_chain_id.try_into().unwrap()]],
        bump,
        seeds::program = GasProgram::id(),)]
    /// Gas Price.
    /// PDA: seeds = [b"price_v2",destination_chain_id], seeds::program = "gas_program"
    /// CHECK: This account holds the gas price data.
    pub gas_price: AccountInfo<'info>,

    #[account(
        seeds = [b"price_v2",&[CHAIN_ID]],
        bump,
        seeds::program = GasProgram::id(),)]
    /// This Gas Price.
    //  PDA: seeds = [b"price_v2",chain_id], seeds::program = "gas_program"
    /// CHECK: This account holds the gas price data.
    pub this_gas_price: AccountInfo<'info>,

    #[account(
        seeds = [b"chain_bridge",&[bridge_to_args.to_chain_id.try_into().unwrap()]],
        bump,
        seeds::program = AllBridgeProgram::id(),)]
    /// Chain Bridge.
    /// PDA: seeds = [b"chain_bridge",destination_chain_id], seeds::program = "allbridge_program"
    /// CHECK: This account is used for chain bridging.
    pub chain_bridge: AccountInfo<'info>,

    /// User Token Account
    /// ATA: mint = mint, owner = payer,
    /// CHECK: This Account is used to send token for user
    pub user_token: Account<'info, TokenAccount>,

    #[account(
        seeds = [config.to_account_info().key.as_ref()],
        bump,
        seeds::program = AllBridgeProgram::id(),)]
    /// Bridge Authority.
    /// PDA: seeds = [config], seeds::program = "AllBridgeProgram"
    /// CHECK: This account is used as the bridge authority.
    pub bridge_authority: AccountInfo<'info>,

    /// Allbridge Program
    /// "BrdgN2RPzEMWF96ZbnnJaUtQDQx7VRXYaHHbYCBvceWB"
    /// CHECK: Allbridge Program ID
    #[account(address = AllBridgeProgram::id())]
    pub allbridge_program: AccountInfo<'info>,

    /// SPL Token Program
    /// "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    /// CHECK: fixed
    #[account(address = Token::id())]
    pub token_program: AccountInfo<'info>,

    /// System Program.
    /// "11111111111111111111111111111111"
    /// CHECK: fixed
    #[account(address = System::id())]
    pub system_program: AccountInfo<'info>,
}

#[cfg(test)]
mod test {
    use super::*;
    use anchor_lang::solana_program::hash;
    pub const SIGHASH_GLOBAL_NAMESPACE: &str = "global";

    #[test]
    pub fn sighash() {
        let name: &str = "swap_and_bridge";
        let preimage = format!("{}:{}", SIGHASH_GLOBAL_NAMESPACE, name);

        let mut sighash = [0u8; 8];
        sighash.copy_from_slice(&hash::hash(preimage.as_bytes()).to_bytes()[..8]);
        msg!("{:?}", sighash);
    }

    #[test]
    fn decode() {
        let hex_string = "c8365927e1a57842834c223b125d5e3c5aac7d26447dee39dafa21f264a59ba1000000000000000000000000c380bedadc4935dd96ddc67dc057bed07738e8af06000000000000000000000000af88d065e77c8cc2239327c5edb3a432268e5831aa0b000000000000";

        match hex::decode(hex_string) {
            Ok(data) => {
                if data.len() != 105 {
                    println!("Data length is incorrect.");
                    return;
                }

                let nonce = &data[0..32];
                let recipient = &data[32..64];
                let destination_chain_id = data[64];
                let receive_token = &data[65..97];
                let vusd_amount = u64::from_le_bytes(data[97..105].try_into().unwrap());

                println!("Decoded successfully:");
                println!("Nonce: {}", bytes_to_hex_string(nonce));
                println!("Recipient: {}", bytes_to_hex_string(recipient));
                println!("Destination Chain ID: {}", destination_chain_id);
                println!("Receive Token: {}", bytes_to_hex_string(receive_token));
                println!("VUSD Amount: {}", vusd_amount);
            }
            Err(e) => println!("Failed to decode hex string: {}", e),
        }
    }

    fn bytes_to_hex_string(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

}
