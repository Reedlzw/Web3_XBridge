use {
    crate::{
        bridge_out::{
            BridgeTo,
            BridgeToArgs,
            BridgeResult,
            SwapType,
        },
        common::{
            DebridgedlnErrorCode,
            debridgedln_program
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
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{spl_token, Mint, TokenAccount}
    },
};

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, BridgeTo<'info>>, data: BridgeToArgs, bridge_to_debridgedln_args: BridgeToDebridgedlnArgs) -> Result<BridgeResult> {

    require!(data.swap_type == SwapType::BRIDGE, DebridgedlnErrorCode::DebridgeDoNotSupportSwapType);
    let bridge_to_debridgedln = BridgeToDebridgedln{
        // account in bridge_to.accounts
        payer: ctx.accounts.payer.clone(),
        mint: ctx.accounts.mint.clone(),
        user_token_account: ctx.accounts.user_token_account.clone(),
        associated_token_program: ctx.accounts.associated_token_program.clone(),
        token_program: ctx.accounts.token_program.clone(),
        system_program: ctx.accounts.system_program.clone(),
        // account in bridge_to.remaining_accounts
        debridge_program: ctx.remaining_accounts[0].to_account_info(),
        state: ctx.remaining_accounts[1].to_account_info(),
        give_order_state: ctx.remaining_accounts[2].to_account_info(),
        authorized_native_sender: ctx.remaining_accounts[3].to_account_info(),
        give_order_wallet: ctx.remaining_accounts[4].to_account_info(),
        nonce_master: ctx.remaining_accounts[5].to_account_info(),
        fee_ledger_wallet: ctx.remaining_accounts[6].to_account_info(),
    };

    // ================================= args ==========================================

    let mut new_data =  vec![130u8, 131u8, 98u8, 190u8, 40u8, 206u8, 68u8, 50u8];

    let order_args_encoded: Vec<u8> = bridge_to_debridgedln_args.order_args.try_to_vec()?;
    new_data.append(&mut order_args_encoded.clone());
    
    if let Some(affiliate_fee) = &bridge_to_debridgedln_args.affiliate_fee {
        let mut affiliate_fee_encoded = affiliate_fee.try_to_vec()?;
        new_data.append(&mut affiliate_fee_encoded);
    } else {
        new_data.push(0u8);
    }

    if let Some(referral_code) = bridge_to_debridgedln_args.referral_code {
        new_data.push(1);
        new_data.extend_from_slice(&referral_code.to_le_bytes());
    }  else {
        new_data.push(0u8);
    }

    new_data.extend_from_slice(&bridge_to_debridgedln_args.nonce.to_le_bytes());

    let mut metadata_encoded = bridge_to_debridgedln_args.metadata.try_to_vec()?;
    let metadata_length = metadata_encoded.len() as u32;
    new_data.extend_from_slice(&metadata_length.to_le_bytes());
    new_data.append(&mut metadata_encoded);

    // ================================= args ==========================================

    let ix = Instruction {
        program_id: debridgedln_program::id(),
        data: new_data,
        accounts:vec![
            AccountMeta::new(bridge_to_debridgedln.payer.key(), true),
            AccountMeta::new_readonly(bridge_to_debridgedln.state.key(), false),
            AccountMeta::new(bridge_to_debridgedln.mint.key(), false),
            AccountMeta::new(bridge_to_debridgedln.give_order_state.key(), false),
            AccountMeta::new_readonly(bridge_to_debridgedln.authorized_native_sender.key(), false),
            AccountMeta::new(bridge_to_debridgedln.user_token_account.key(), false),
            AccountMeta::new(bridge_to_debridgedln.give_order_wallet.key(), false),
            AccountMeta::new(bridge_to_debridgedln.nonce_master.key(), false),
            AccountMeta::new(bridge_to_debridgedln.fee_ledger_wallet.key(), false),
            AccountMeta::new_readonly(bridge_to_debridgedln.system_program.key(), false),
            AccountMeta::new_readonly(bridge_to_debridgedln.token_program.key(), false),
            AccountMeta::new_readonly(bridge_to_debridgedln.associated_token_program.key(), false),
        ],
    };

    invoke(
        &ix,
        &bridge_to_debridgedln.to_account_infos()
    )?;
        

    Ok(BridgeResult{
        ext: "".to_string(),
    })
}


#[derive(Debug, AnchorDeserialize, AnchorSerialize, Clone)]
pub struct BridgeToDebridgedlnArgs {
    pub order_args: CreateOrderArgs,
    pub affiliate_fee: Option<AffiliateFee>,
    pub referral_code: Option<u32>,
    pub nonce: u64,
    pub metadata: Vec<u8>,
    pub orderid: Vec<u8>,
}

impl BridgeToDebridgedlnArgs {
    pub fn try_from_vec(data: &[u8]) -> Result<BridgeToDebridgedlnArgs> {
        let decoded_args = BridgeToDebridgedlnArgs::try_from_slice(&data)?;
        Ok(decoded_args)
    }
}


#[derive(Debug, AnchorDeserialize, AnchorSerialize, Clone)]
pub struct CreateOrderArgs {
    pub give_original_amount: u64,
    pub take: Offer,
    pub receiver_dst: Vec<u8>,
    pub external_call: Option<Vec<u8>>,
    pub give_patch_authority_src: Pubkey,
    pub allowed_cancel_beneficiary_src: Option<Pubkey>,
    pub order_authority_address_dst: Vec<u8>,
    pub allowed_taker_dst: Option<Vec<u8>>,
}

#[derive(Debug, AnchorDeserialize, AnchorSerialize, Clone)]
pub struct AffiliateFee {
    pub beneficiary: Option<Pubkey>,
    pub amount: Option<u64>,
}

#[derive(Debug, AnchorDeserialize, AnchorSerialize, Clone)]
pub struct Offer {
    pub chain_id: [u8; 32],
    pub token_address: Vec<u8>,
    pub amount: [u8; 32],
}

// #[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone)]
// pub struct ExternalBridgeToDebridgedlnArgs {
//     pub order_args: CreateOrderArgs,
//     pub affiliate_fee: Option<AffiliateFee>,
//     pub referral_code: Option<u32>,
//     pub nonce: u64,
//     pub metadata: Vec<u8>,
// }

#[derive(Accounts)]
#[instruction(bridge_to_debridgedln_args: BridgeToDebridgedlnArgs)]
pub struct BridgeToDebridgedln<'info> {

    #[account(mut)]
    /// order maker, pays dln fix fee
    pub payer: Signer<'info>,

    /// debridge Program
    /// "src5qyZHqTqecJV4aY6Cb6zDZLMDzrDKKezs22MPHr4"
    /// CHECK: Debridge Program ID
    #[account(address = debridgedln_program::id())]
    pub debridge_program: AccountInfo<'info>,

    #[account(
        seeds = [b"STATE".as_ref()],
        bump,
        seeds::program = debridge_program.key()
    )]
    /// state
    /// PDA: seeds = ["STATE"], program = DLN_SRC
    /// CHECK:
    pub state: AccountInfo<'info>,

    #[account(mut)]
    /// tokenMint
    /// give token mint
    /// CHECK:
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"GIVE_ORDER_STATE".as_ref(), &bridge_to_debridgedln_args.orderid.as_ref()],
        bump,
        seeds::program = debridge_program.key(),
    )]
    // #[account(mut)]
    /// giveOrderState
    /// Account with GiveOrderState
    /// Will be initialized inside [`create_order`]
    /// PDA: seeds = ["GIVE_ORDER_STATE", order_id], program = DLN_SRC
    /// CHECK:
    pub give_order_state: AccountInfo<'info>,

    #[account(
        seeds = ["AUTHORIZED_NATIVE_SENDER".as_ref(), &bridge_to_debridgedln_args.order_args.take.chain_id.as_ref()],
        bump,
        seeds::program = debridge_program.key(),
    )]
    /// authorizedNativeSender
    /// PDA: seeds = ["AUTHORIZED_NATIVE_SENDER", chainId as [u8; 32]], program = DLN_SRC
    /// CHECK: 
    pub authorized_native_sender: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = payer,
    )]
    /// makerWallet
    /// wallet to transfer give token from, owned by maker, usually ATA(tokenMint, maker)
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"GIVE_ORDER_WALLET".as_ref(), &bridge_to_debridgedln_args.orderid.as_ref()],
        bump,
        seeds::program = debridge_program.key(),
    )]
    // #[account(mut)]
    /// giveOrderWallet
    /// PDA: seeds = ["GIVE_ORDER_WALLET", order_id], program = DLN_SRC
    /// "Wallet of `give_order_state`", "Will be initialized inside [`create_order`]"
    /// CHECK:
    pub give_order_wallet: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"NONCE".as_ref(), payer.key().as_ref()],
        bump,
        seeds::program = debridge_program.key(),
    )]
    // #[account(mut)]
    /// nonceMaster
    /// PDA: seeds = ["NONCE", maker], program = DLN_SRC
    /// CHECK:
    pub nonce_master: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"FEE_LEDGER_WALLET".as_ref(), mint.key().as_ref()],
        bump,
        seeds::program = debridge_program.key(),
    )]
    // #[account(mut)]
    /// feeLedgerWallet
    /// PDA: seeds = ["FEE_LEDGER_WALLET", tokenMint], program = DLN_SRC
    /// CHECK:
    pub fee_ledger_wallet: AccountInfo<'info>,

    /// systemProgram.
    /// CHECK: fixed
    #[account(address = system_program::id())]
    pub system_program: AccountInfo<'info>,

    /// splTokenProgram
    /// CHECK: fixed
    #[account(address = spl_token::id())]
    pub token_program: AccountInfo<'info>,

    /// associatedSplTokenProgram
    /// CHECK: fixed
    #[account(address = AssociatedToken::id())]
    pub associated_token_program: Program<'info, AssociatedToken>,
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test1() {
        let new_data =  vec![130u8, 131u8, 98u8, 190u8, 40u8, 206u8, 68u8, 50u8, 197u8, 78u8, 51u8];
        msg!("{:?}", new_data);
    }

    #[test]
    fn decode() {
        // let hex_string = "d786410000000000000000000000000000000000000000000000000000000000000000000000a4b114000000af88d065e77c8cc2239327c5edb3a432268e583100000000000000000000000000000000000000000000000000000000002db956140000008b3997e0a91ddf63585abbc032c406f47ad456330006e2656c7b84f950802bcb474c4f5b32c0c5148cfa8c1e86410bf6cbdd3183db00140000008b3997e0a91ddf63585abbc032c406f47ad456330000006e7dd2b78f0100004500000041000000010101000017c013000000000000000000000000000000000056b92d00000000000000000000000000000000000000000000000000000000000000000000000000";
        
        // match hex::decode(hex_string) {
        //     Ok(data) => {
        //         match ExternalBridgeToDebridgedlnArgs::try_from_slice(&data) {
        //             Ok(decoded_args) => println!("Decoded successfully: {:?}", decoded_args),
        //             Err(e) => println!("Failed to deserialize: {:?}", e),
        //         }
        //     },
        //     Err(e) => println!("Failed to decode hex string: {}", e),
        // }


        // let hex_string = "1492410000000000000000000000000000000000000000000000000000000000000000000000a4b114000000af88d065e77c8cc2239327c5edb3a432268e583100000000000000000000000000000000000000000000000000000000002db954140000008b3997e0a91ddf63585abbc032c406f47ad456330006e2656c7b84f950802bcb474c4f5b32c0c5148cfa8c1e86410bf6cbdd3183db00140000008b3997e0a91ddf63585abbc032c406f47ad45633000000333cc4b78f01000041000000010101000054cb13000000000000000000000000000000000054b92d000000000000000000000000000000000000000000000000000000000000000000000000002000000089d71af6dbd77e8ae478ae2333989e0d9a6c82055e1feebb5fcc49c46917c806";
        
        // match hex::decode(hex_string) {
        //     Ok(data) => {
        //         match BridgeToDebridgedlnArgs::try_from_slice(&data) {
        //             Ok(decoded_args) => println!("Decoded successfully: {:?}", decoded_args),
        //             Err(e) => println!("Failed to deserialize: {:?}", e),
        //         }
        //     },
        //     Err(e) => println!("Failed to decode hex string: {}", e),
        // }
    }

    #[test]
    fn decode_from_buffer() {
        let data: Vec<u8> = vec![
            34u8, 32u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 139u8, 57u8, 151u8, 224u8, 169u8, 29u8, 223u8, 99u8, 88u8, 90u8, 187u8, 192u8, 50u8, 196u8, 6u8, 244u8, 122u8, 212u8, 86u8, 51u8, 199u8, 39u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 177u8, 164u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 96u8, 6u8, 65u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 38u8, 1u8, 0u8, 0u8, 65u8, 6u8, 96u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 164u8, 177u8, 20u8, 0u8, 0u8, 0u8, 175u8, 136u8, 208u8, 101u8, 231u8, 124u8, 140u8, 194u8, 35u8, 147u8, 39u8, 197u8, 237u8, 179u8, 164u8, 50u8, 38u8, 142u8, 88u8, 49u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 45u8, 185u8, 113u8, 20u8, 0u8, 0u8, 0u8, 139u8, 57u8, 151u8, 224u8, 169u8, 29u8, 223u8, 99u8, 88u8, 90u8, 187u8, 192u8, 50u8, 196u8, 6u8, 244u8, 122u8, 212u8, 86u8, 51u8, 0u8, 6u8, 226u8, 101u8, 108u8, 123u8, 132u8, 249u8, 80u8, 128u8, 43u8, 203u8, 71u8, 76u8, 79u8, 91u8, 50u8, 192u8, 197u8, 20u8, 140u8, 250u8, 140u8, 30u8, 134u8, 65u8, 11u8, 246u8, 203u8, 221u8, 49u8, 131u8, 219u8, 0u8, 20u8, 0u8, 0u8, 0u8, 139u8, 57u8, 151u8, 224u8, 169u8, 29u8, 223u8, 99u8, 88u8, 90u8, 187u8, 192u8, 50u8, 196u8, 6u8, 244u8, 122u8, 212u8, 86u8, 51u8, 0u8, 0u8, 0u8, 169u8, 38u8, 253u8, 169u8, 143u8, 1u8, 0u8, 0u8, 65u8, 0u8, 0u8, 0u8, 1u8, 1u8, 1u8, 0u8, 0u8, 160u8, 63u8, 19u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 113u8, 185u8, 45u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 32u8, 0u8, 0u8, 0u8, 187u8, 210u8, 61u8, 89u8, 117u8, 16u8, 211u8, 184u8, 46u8, 29u8, 183u8, 221u8, 191u8, 111u8, 201u8, 51u8, 137u8, 36u8, 92u8, 105u8, 26u8, 217u8, 163u8, 244u8, 100u8, 90u8, 64u8, 51u8, 92u8, 15u8, 239u8, 167u8, 0u8, 0u8, 0u8, 0u8
        ];

        match BridgeToDebridgedlnArgs::try_from_slice(&data) {
            Ok(decoded_args) => {
                println!("Decoded successfully: {:?}", decoded_args);
            },
            Err(e) => println!("Failed to deserialize: {:?}", e),
        }
    }
}