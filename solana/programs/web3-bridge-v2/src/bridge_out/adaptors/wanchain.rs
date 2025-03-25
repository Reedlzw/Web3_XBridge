use {
    crate::{
        bridge_out::{
            BridgeTo,
            BridgeToArgs,
            BridgeResult,
        },
        common::{
            wanchain_sol_value,
            wanchain_fee_receiver,
            wanchain_admin_board_program,
            wanchain_config_account,
            wanchain_circle_config_program,
            wanchain_program
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

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, BridgeTo<'info>>, data: BridgeToArgs, bridge_to_wanchain_args: BridgeToWanchainArgs) -> Result<BridgeResult> {

    let bridge_to_wanchain = BridgeToWanchain{
        // account in bridge_to.accounts
        payer: ctx.accounts.payer.clone(),
        user_ata: ctx.accounts.user_token_account.clone(),
        mapping_token_mint: ctx.accounts.mint.clone(),
        associated_token_program: ctx.accounts.associated_token_program.clone(),
        token_program: ctx.accounts.token_program.clone(),
        system_program: ctx.accounts.system_program.clone(),
        // account in bridge_to.remaining_accounts
        wanchain_program: ctx.remaining_accounts[0].to_account_info(),
        sol_vault: ctx.remaining_accounts[1].to_account_info(),
        token_vault: ctx.remaining_accounts[2].to_account_info(),
        fee_receiver: ctx.remaining_accounts[3].to_account_info(),
        admin_board_program: ctx.remaining_accounts[4].to_account_info(),
        config_account: ctx.remaining_accounts[5].to_account_info(),
        token_pair_account: ctx.remaining_accounts[6].to_account_info(),
        cctp_admin_board_fee_account: ctx.remaining_accounts[7].to_account_info(),
    };

    let mut new_data =  vec![66u8, 17u8, 214u8, 126u8, 235u8, 133u8, 82u8, 114u8];
    // Append smg_id (32 bytes)
    new_data.extend_from_slice(&bridge_to_wanchain_args.smg_id);
    // Append token_pair_id (u32, little-endian)
    new_data.extend_from_slice(&bridge_to_wanchain_args.token_pair_id.to_le_bytes());
    // Append amount (u64, little-endian)
    new_data.extend_from_slice(&data.amount.to_le_bytes());
    // Append amount (bytes)
    let to_address = &data.to;
    let address_bytes = &to_address[to_address.len() - 20..];       // for evm address
    let mut formatted_address = Vec::new();
    formatted_address.extend_from_slice("0x".as_bytes());
    for byte in address_bytes {
        formatted_address.extend_from_slice(format!("{:02X}", byte).as_bytes());
    }
    new_data.extend_from_slice(&(formatted_address.len() as u32).to_le_bytes());
    new_data.extend_from_slice(&formatted_address);


    let ix = Instruction{
        program_id: wanchain_program::id(),
        data: new_data,
        accounts:vec![
            AccountMeta::new(bridge_to_wanchain.payer.key(), true),
            AccountMeta::new(bridge_to_wanchain.sol_vault.key(), false),
            AccountMeta::new(bridge_to_wanchain.user_ata.key(), false),
            AccountMeta::new(bridge_to_wanchain.token_vault.key(), false),
            AccountMeta::new(bridge_to_wanchain.mapping_token_mint.key(), false),
            AccountMeta::new(bridge_to_wanchain.fee_receiver.key(), false),
            AccountMeta::new_readonly(bridge_to_wanchain.admin_board_program.key(), false),
            AccountMeta::new_readonly(bridge_to_wanchain.config_account.key(), false),
            AccountMeta::new(bridge_to_wanchain.token_pair_account.key(), false),
            AccountMeta::new_readonly(bridge_to_wanchain.cctp_admin_board_fee_account.key(), false),
            AccountMeta::new_readonly(bridge_to_wanchain.token_program.key(), false),
            AccountMeta::new_readonly(bridge_to_wanchain.associated_token_program.key(), false),
            AccountMeta::new_readonly(bridge_to_wanchain.system_program.key(), false),
        ],
    };

    invoke(
        &ix,
        &bridge_to_wanchain.to_account_infos()
    )?;


    Ok(BridgeResult{
        ext: "".to_string(),
    })
}

#[derive(Accounts)]
#[instruction(bridge_to_wanchain_args: BridgeToWanchainArgs)]
pub struct BridgeToWanchain<'info> {
    #[account(mut)]
    /// payer
    pub payer: Signer<'info>,

    /// wanchin Program
    /// "E3iKvJgGNycXrmsh2aryY25z29PpU4dvo4CBuXCKQiGB"
    /// CHECK: Wanchain Program ID
    #[account(address = wanchain_program::id())]
    pub wanchain_program: AccountInfo<'info>,

    #[account(
        mut,
        address = wanchain_sol_value::id()
    )]
    /// solValut
    /// CHECK:
    pub sol_vault: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::mint = mapping_token_mint,
        associated_token::authority = payer,
    )]
    /// userAta
    pub user_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        // associated_token::mint = mapping_token_mint,
        // associated_token::authority = sol_vault,
    )]
    /// tokenVault 
    /// CHECK:
    pub token_vault: AccountInfo<'info>,

    #[account(mut)]
    /// mappingTokenMint    // 跨链币种 mint地址
    pub mapping_token_mint: Account<'info, Mint>,

    #[account(
        mut,
        address = wanchain_fee_receiver::id()
    )]
    /// feeReceiver
    /// CHECK:
    pub fee_receiver: AccountInfo<'info>,

    #[account(
        address = wanchain_admin_board_program::id()
    )]
    /// adminBoardProgram
    /// CHECK:
    pub admin_board_program: AccountInfo<'info>,

    #[account(
        address = wanchain_config_account::id()
    )]
    /// configAccount
    /// CHECK:
    pub config_account: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [b"TokenPairInfo", &bridge_to_wanchain_args.token_pair_id.to_le_bytes()],
        bump,
        seeds::program = admin_board_program.key()
    )]
    /// tokenPairAccount
    /// CHECK:
    pub token_pair_account: AccountInfo<'info>,

    #[account(
        seeds = [b"FeeData", &bridge_to_wanchain_args.slip44_chain_id.to_le_bytes()],
        bump,
        seeds::program = wanchain_circle_config_program::id()
    )]
    /// cctpAdminBoardFeeAccount
    /// CHECK: 
    pub cctp_admin_board_fee_account: AccountInfo<'info>,

    #[account(address = spl_token::id())]
    /// tokenProgram
    /// CHECK:
    pub token_program: AccountInfo<'info>,

    #[account(address = AssociatedToken::id())]
    /// associatedSplTokenProgram
    /// CHECK: 
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(address = system_program::id())]
    /// systemProgram.
    /// CHECK:
    pub system_program: AccountInfo<'info>,
}

#[derive(Debug, AnchorDeserialize, AnchorSerialize, Clone)]
pub struct BridgeToWanchainArgs {
    pub smg_id: [u8; 32],
    pub token_pair_id: u32,
    pub slip44_chain_id: u32,
}

impl BridgeToWanchainArgs {
    pub fn try_from_vec(data: &[u8]) -> Result<BridgeToWanchainArgs> {
        let decoded_args = BridgeToWanchainArgs::try_from_slice(&data)?;
        Ok(decoded_args)
    }
}