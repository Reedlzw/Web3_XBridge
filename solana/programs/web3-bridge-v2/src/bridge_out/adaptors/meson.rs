use {
    crate::{
        bridge_out::{
            BridgeResult, BridgeTo, BridgeToArgs, SwapType
        },
        common::{
            meson_program as Meson, vec_to_hex_string, MesonErrorCode
        }
    },
    anchor_lang::{
        prelude::*, 
        solana_program::{
            instruction::Instruction, program::invoke, system_program
        }
    },
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{spl_token, Mint, TokenAccount}
    },
    serde::Serialize,
};

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, BridgeTo<'info>>, data: BridgeToArgs, bridge_to_meson_args: BridgeToMesonArgs) -> Result<BridgeResult> {

    require!(data.swap_type == SwapType::BRIDGE, MesonErrorCode::MesonDoNotSupportSwapType);
    // concat data from BridgeToMesonArgs to vec<u8>
    let mut new_data =  vec![LegacyInstruction::PostSwapFromInitiator as u8];
    let mut encoded_clone = bridge_to_meson_args.encoded.clone();
    let mut initiator_clone = bridge_to_meson_args.initiator.clone();
    new_data.append(&mut encoded_clone);
    new_data.append(&mut initiator_clone);
    new_data.append(&mut vec![00u8,00u8,00u8,00u8,00u8,00u8,00u8,01u8]);
    let encoded_clone = bridge_to_meson_args.encoded.clone();
    let initiator_clone = bridge_to_meson_args.initiator.clone();


    let bridge_to_meson = BridgeToMeson{
        // account in bridge_to.accounts
        payer: ctx.accounts.payer.clone(),
        mint: ctx.accounts.mint.clone(),
        user_token_account: ctx.accounts.user_token_account.clone(),
        associated_token_program: ctx.accounts.associated_token_program.clone(),
        token_program: ctx.accounts.token_program.clone(),
        system_program: ctx.accounts.system_program.clone(),
        // account in bridge_to.remaining_accounts
        meson_contract_signer: ctx.remaining_accounts[0].to_account_info(),
        meson_token_account: ctx.remaining_accounts[1].to_account_info(),
        supported_token_account: ctx.remaining_accounts[2].to_account_info(),
        posted_token_account: ctx.remaining_accounts[3].to_account_info(),
        meson_program: ctx.remaining_accounts[4].to_account_info(),
    };

    let ix: Instruction = Instruction {
            program_id: Meson::id(),
            data: new_data,
            accounts:vec![
                AccountMeta::new_readonly(bridge_to_meson.system_program.key(), false), // fixed 
                AccountMeta::new_readonly(bridge_to_meson.token_program.key(), false),  // fixed
                AccountMeta::new(bridge_to_meson.meson_token_account.key(), false),  // depend on meson
                AccountMeta::new(bridge_to_meson.mint.key(), false), // usdc or usdt
                AccountMeta::new(bridge_to_meson.meson_contract_signer.key(), false),  // depend on meson
                AccountMeta::new(bridge_to_meson.supported_token_account.key(), false), // support 
                AccountMeta::new(bridge_to_meson.posted_token_account.key(), false), // posted
                // AccountMeta::new(posted_token_account_pubkey, false), // posted
                AccountMeta::new(bridge_to_meson.payer.key(), true),
                AccountMeta::new(bridge_to_meson.user_token_account.key(), false),
            ],
    };

    invoke(
        &ix,
        &bridge_to_meson.to_account_infos()
    )?;

    let meson_log = MesonExtLog{
        encoded: "0x".to_string() + &vec_to_hex_string(encoded_clone),
        initiator: "0x".to_string() + &vec_to_hex_string(initiator_clone),
    };

    Ok(BridgeResult{
        ext: serde_json::to_string(&meson_log).unwrap(),
    })
}


#[derive(AnchorDeserialize, AnchorSerialize, Clone, Debug)]
pub struct BridgeToMesonArgs {
    /// encoded.
    /// From: https://relayer.meson.fi/api/v1/swap
    pub encoded: Vec<u8>,
    /// initiator.
    /// From: https://relayer.meson.fi/api/v1/swap
    pub initiator: Vec<u8>,
}

impl BridgeToMesonArgs{
    pub fn try_from_vec(data: &[u8]) -> Result<BridgeToMesonArgs> {
        let encoded = data[0..32].to_vec();
        let initiator = data[32..52].to_vec();
        Ok(BridgeToMesonArgs{
            encoded,
            initiator,
        })
    }
}

#[derive(Accounts)]
#[instruction(bridge_to_meson_args: BridgeToMesonArgs)]
pub struct BridgeToMeson<'info> {
    #[account(mut)]
    /// Payer.
    /// This account is user to pay for the transaction and bridge token.
    /// CHECK: This account is user to pay for the transaction and bridge token.
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"contract_signer".as_ref()],
        bump,
        seeds::program = meson_program.key(),
    )]
    /// Meson Contract Signer.
    /// PDA: seeds = [b"contract_signer"], seeds::program = "meson_program"
    /// CHECK: This Account is used to create ATA account to receive token for meson.
    pub meson_contract_signer: AccountInfo<'info>,

    /// CHECK: Mint (read-only).
    pub mint: Account<'info, Mint>,
    
    #[account(
        mut,
        // associated_token::mint = mint,
        // associated_token::authority = meson_contract_signer,
    )]
    /// Meson Token Account.
    /// ATA: mint = mint, owner = meson_contract_signer,
    /// CHECK: This Account is used to receive token for meson.
    // pub meson_token_account: Account<'info, TokenAccount>,
    pub meson_token_account: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = payer,
    )]
    /// User Token Account
    /// ATA: mint = mint, owner = payer,
    /// CHECK: This Account is used to send token for user,
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"supported_tokens".as_ref()],
        bump,
        seeds::program = meson_program.key(),
    )]
    /// Supported Token Account
    /// PDA: seeds = [b"supported_tokens"], seeds::program = "meson_program".
    /// CHECK: This account is a PDA account, which is used to record support token for meson.
    pub supported_token_account: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [ b"posted_swap".as_ref(), bridge_to_meson_args.encoded.as_ref()],
        bump,
        seeds::program = meson_program.key(),
    )]
    /// Posted Token Account
    /// PDA: seeds = [b"posted_swap", encoded], seeds::program = "meson_program".
    /// CHECK: This account is a PDA account, which is used to record posted record for meson.
    pub posted_token_account: AccountInfo<'info>,

    /// Meson Program
    /// "FR1SDyLUj7PrMbtkUCkDrBymk5eWrRmr3UvWFb5Kjbmd"
    /// CHECK: Meson Program ID
    #[account(address = Meson::id())]
    pub meson_program: AccountInfo<'info>,

    /// Associated Token Program
    /// "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
    /// CHECK: fixed
    // #[account(address = AssociatedToken::id())]
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// SPL Token Program
    /// "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    /// CHECK: fixed
    #[account(address = spl_token::id())]
    pub token_program: AccountInfo<'info>,
    // pub token_program: Program<'info, Token>,

    /// System Program.
    /// "11111111111111111111111111111111"
    /// CHECK: fixed
    #[account(address = system_program::id())]
    pub system_program: AccountInfo<'info>,
    // pub system_program: Program<'info, System>,
}

/// NOTE: No more instructions should be added to this enum. Instead, add them as Anchor instruction
/// handlers, which will inevitably live in lib.rs.
#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone)]
pub enum LegacyInstruction {
  /* 00 */ Init,
  /* 01 */ TransferOwnership,
  /* 02 */ TransferPremiumManager,
  /* 03 */ AddSupportToken,
  /* 04 */ DepositAndRegister,
  /* 05 */ Deposit,
  /* 06 */ Withdraw,
  /* 07 */ AddAuthorizedAddr,
  /* 08 */ RemoveAuthorizedAddr,
  /* 09 */ TransferPoolOwner,
  /* 10 */ WithdrawServiceFee,
  /* 11 */ PostSwapFromInitiator,
  /* 12 */ BondSwap,
  /* 13 */ CancelSwap,
  /* 14 */ ExecuteSwap,
  /* 15 */ LockSwap,
  /* 16 */ Unlock,
  /* 17 */ Release,
  /* 18 */ RirectRelease,
}

#[event]
pub struct CrossChainDataEvent{
    adaptor_id: u8,
    swap_type: u8,
    amount: u64,
    encoded: String,
    initiator: String,
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Serialize)]
pub struct MesonExtLog{
    encoded: String,
    initiator: String,
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Encoded{
    version: u8,
    amount: u64,
    salt_header: String, 
    salt_data: String,
    fee: u64,
    expire_ts: u64,
    in_chain: String,
    in_token: u8,
    out_chain: String,
    out_token: u8,
}

impl Encoded{
    pub fn try_to_decode(encoded: Vec<u8>) -> Result<Encoded> {
        let mut amount = [0u8; 8];
        amount[3..8].copy_from_slice(&encoded[1..6]);

        let mut fee = [0u8; 8];
        fee[3..8].copy_from_slice(&encoded[16..21]);

        let mut expire_ts = [0u8; 8];
        expire_ts[3..8].copy_from_slice(&encoded[21..26]);

        let mut out_token = [0u8; 1];
        out_token[0..1].copy_from_slice(&encoded[28..29]);

        let mut in_token = [0u8; 1];
        in_token[0..1].copy_from_slice(&encoded[31..32]);

        Ok(Encoded{
            version: encoded[0..1][0],
            amount: u64::from_be_bytes(amount),
            salt_header: vec_to_hex_string(encoded[6..8].to_vec()),
            salt_data: vec_to_hex_string(encoded[8..16].to_vec()),
            fee: u64::from_be_bytes(fee),
            expire_ts: u64::from_be_bytes(expire_ts),
            out_chain: vec_to_hex_string(encoded[26..28].to_vec()),
            out_token: u8::from_be_bytes(out_token),
            in_chain: vec_to_hex_string(encoded[29..31].to_vec()),
            in_token: u8::from_be_bytes(in_token),
        })
    }

    pub fn try_to_change_amount(mut encoded: Vec<u8>, amount: [u8; 8]) -> Vec<u8> {
        encoded[1..6].copy_from_slice(&amount[3..8]);
        encoded
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn decode_data_data(){
        let _data_data:Vec<u8> = vec![1, 0, 0, 76, 75, 64, 152, 0, 0, 0, 0, 0, 133, 91, 125, 30, 0, 0, 23, 205, 192, 0, 102, 38, 170, 142, 3, 198, 1, 1, 245, 1, 48, 251, 56, 214, 113, 5, 9, 115, 122, 122, 26, 118, 51, 122, 22, 193, 164, 170, 56, 22];
        // data_data
        let _amount: u64 = 1234566;
        // Encoded::try_to_change_amount(data_data, amount.to_be_bytes());
        // let _data = BridgeToMesonArgs::try_from_vec(data_data).unwrap();
        // msg!("{:?}", helper::vec_to_hex_string(_data.encoded));
        // msg!("{:?}", helper::vec_to_hex_string(_data.initiator));

    }
    #[test]
    fn concat_vec(){
        let mut a = vec![1u8,2u8,3u8];
        let mut b = vec![4u8,5u8,6u8];

        a.append(&mut b);
        msg!("a: {:?}", a);
        msg!("res: {:?}", (a, b).try_to_vec());
    }

    #[test]
    fn decode_encoded() {
        
        // 0x01,0x00,0x00,0x1e,0x84,0x80,0x98,0x00,0x00,0x00,0x00,0x00,0xe1,0x69,0x09,0xa3,0x00,0x00,0x00,0x27,0x10,0x00,0x65,0xb2,0x57,0x33,0x03,0xc6,0x01,0x01,0xf5,0x01
        
        let encoded = vec![01u8, 00u8, 00u8, 30u8, 132u8, 128u8, 152u8, 00u8, 00u8, 00u8, 00u8, 00u8, 225u8, 105u8, 09u8, 163u8, 00u8, 00u8, 00u8, 39u8, 16u8, 00u8, 101u8, 178u8, 87u8, 51u8, 03u8, 198u8, 01u8, 01u8, 245u8, 01u8];
        let _encoded_clone = encoded.clone();
        let event = CrossChainDataEvent{
            adaptor_id: 1,
            swap_type: 0,
            amount: 30,
            // encoded: "0x".to_string() + &helper::vec_to_hex_string(encoded.clone()),
            encoded: "0x".to_string(),
            initiator: "0x".to_string(),
        };
        emit!(event);
        // source code from meson sdk
        // const version = parseInt(`0x${encoded.substring(2, 4)}`, 16)
        // const amount = BigNumber.from(`0x${encoded.substring(4, 14)}`)
        // const saltHeader = `0x${encoded.substring(14, 18)}`
        // const saltData = `0x${encoded.substring(18, 34)}`
        // const fee = BigNumber.from(`0x${encoded.substring(34, 44)}`)
        // const expireTs = parseInt(`0x${encoded.substring(44, 54)}`, 16)
        // const outChain = `0x${encoded.substring(54, 58)}`
        // const outToken = parseInt(`0x${encoded.substring(58, 60)}`, 16)
        // const inChain = `0x${encoded.substring(60, 64)}`
        // const inToken = parseInt(`0x${encoded.substring(64, 66)}`, 16)
        let encoded_clone = encoded.clone();
        let res = Encoded::try_to_decode(encoded).unwrap();
        print!("res: {:?}", res);
        print!("\n");
        let new_amoount = 100u64;
        let encoded_change = Encoded::try_to_change_amount(encoded_clone, new_amoount.to_be_bytes());
        let res_encoded_change = Encoded::try_to_decode(encoded_change).unwrap();
        print!("\nres: {:?}", res_encoded_change);
    }
}