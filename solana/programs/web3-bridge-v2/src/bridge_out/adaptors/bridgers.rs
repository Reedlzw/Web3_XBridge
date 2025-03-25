use {
    crate::{
        bridge_out::{
            BridgeResult, BridgeTo, BridgeToArgs
        },
        common::{
            bridgers_program, bridgers_vs_info, wrapped_sol,
            BridgersErrorCode
        }
    },
    anchor_lang::{
        prelude::*, 
        solana_program::{
            instruction::Instruction, program::invoke, sysvar
        }
    },
    anchor_spl::token::{Token, Mint, TokenAccount}
};



#[derive(AnchorDeserialize, AnchorSerialize, Clone, Debug)]
pub struct BridgeToBridgersArgs {
    pub _selector_id: u8,
    pub _from_token: Vec<u8>,
    pub _sender: Vec<u8>,
    pub _min_return_amount: Vec<u8>,
    pub _to_token: Vec<u8>,
    pub _destination: Vec<u8>,
}
impl BridgeToBridgersArgs {
    pub fn try_from_vec(data: &[u8]) -> Result<BridgeToBridgersArgs> {
        let decoded_args = BridgeToBridgersArgs::try_from_slice(&data)?;
        Ok(decoded_args)
    }
}

#[derive(Accounts)]
pub struct BridgeToBridgersSpl<'info> {
    #[account(mut)]
    /// source_token_info
    /// payer token ata address
    /// CHECK: payer token ata address
    pub source_token_info: Account<'info, TokenAccount>,

    /// payer
    /// CHECK: payer
    pub source_token_auth: Signer<'info>,

    /// SPL Token Program
    /// "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    /// CHECK: fixed
    #[account(address = Token::id())]
    pub token_program: AccountInfo<'info>,

    #[account(
        mut,
        // associated_token::mint = ,
        // associated_token::authority = bridgers_dest_owner::ID,
    )]
    /// dest_token_info
    /// bridgers receive ata address
    /// CHECK: bridgers receive ata address
    pub dest_token_info: AccountInfo<'info>,

    /// vs_info
    /// bridgers_vs_info
    /// CHECK: bridgers_vs_info
    #[account(address = bridgers_vs_info::id())]
    pub vs_info: AccountInfo<'info>,

    /// Bridgers Program
    /// "FDF8AxHB8UK7RS6xay6aBvwS3h7kez9gozqz14JyfKsg"
    /// CHECK: Bridgers Program ID
    #[account(address = bridgers_program::id())]
    pub bridgers_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct BridgeToBridgersSol<'info> {

    #[account(mut)]
    /// payer
    /// CHECK: payer
    pub payer_account: Signer<'info>,

    /// SPL Token Program
    /// "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
    /// CHECK: fixed
    #[account(address = Token::id())]
    pub token_program: AccountInfo<'info>,

    #[account(
        mut,
        // associated_token::mint = ,
        // associated_token::authority = bridgers_dest_owner::ID,
    )]
    /// dest_token_info
    /// bridgers receive ata address
    /// CHECK: bridgers receive ata address
    pub dest_token_info: AccountInfo<'info>,

    /// vs_info
    /// bridgers_vs_info
    /// CHECK: bridgers_vs_info
    #[account(address = bridgers_vs_info::id())]
    pub vs_info: AccountInfo<'info>,

    /// rent Program
    /// "SysvarRent111111111111111111111111111111111"
    /// CHECK: fixed
    #[account(address = sysvar::rent::ID)]
    pub rent_sysvar: AccountInfo<'info>,

    /// System Program.
    /// "11111111111111111111111111111111"
    /// CHECK: fixed
    #[account(address = System::id())]
    pub system_program: AccountInfo<'info>,

    /// mint account info
    /// "So11111111111111111111111111111111111111112"
    /// CHECK: wsol
    #[account(address = wrapped_sol::id())]
    pub mint_account_info: Account<'info, Mint>,

    /// create new account
    /// CHECK: fixed
    #[account(mut)]
    pub pda_account_info: AccountInfo<'info>,

    /// Bridgers Program
    /// "FDF8AxHB8UK7RS6xay6aBvwS3h7kez9gozqz14JyfKsg"
    /// CHECK: Bridgers Program ID
    #[account(address = bridgers_program::id())]
    pub bridgers_program: AccountInfo<'info>,
}



pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, BridgeTo<'info>>, data: BridgeToArgs, bridge_to_bridgers_args: BridgeToBridgersArgs) -> Result<BridgeResult> {

    let mut new_data = Vec::new();
    new_data.push(bridge_to_bridgers_args._selector_id);
    new_data.extend_from_slice(&data.amount.to_le_bytes()); 
    let from_token_len = (bridge_to_bridgers_args._from_token.len() as u32).to_le_bytes();
    new_data.extend_from_slice(&from_token_len);
    new_data.extend(bridge_to_bridgers_args._from_token.clone());
    let sender_len = (bridge_to_bridgers_args._sender.len() as u32).to_le_bytes();
    new_data.extend_from_slice(&sender_len);
    new_data.extend(bridge_to_bridgers_args._sender.clone());
    let min_return_len = (bridge_to_bridgers_args._min_return_amount.len() as u32).to_le_bytes();
    new_data.extend_from_slice(&min_return_len);
    new_data.extend(bridge_to_bridgers_args._min_return_amount.clone());
    let to_token_len = (bridge_to_bridgers_args._to_token.len() as u32).to_le_bytes();
    new_data.extend_from_slice(&to_token_len);
    new_data.extend(bridge_to_bridgers_args._to_token.clone());
    let destination_len = (bridge_to_bridgers_args._destination.len() as u32).to_le_bytes();
    new_data.extend_from_slice(&destination_len);
    new_data.extend(bridge_to_bridgers_args._destination.clone());

    match bridge_to_bridgers_args._selector_id {
        0x02 => {
            msg!("Processing SPL Token Transfer");
            let bridge_to_bridgers_spl = BridgeToBridgersSpl{
                source_token_info: ctx.accounts.user_token_account.clone(),
                source_token_auth: ctx.accounts.payer.clone(),
                token_program: ctx.accounts.token_program.clone(),
                dest_token_info: ctx.remaining_accounts[1].to_account_info(),
                vs_info: ctx.remaining_accounts[2].to_account_info(),
                bridgers_program: ctx.remaining_accounts[0].to_account_info()
            };
            let ix_spl = Instruction {
                program_id: bridgers_program::id(),
                data: new_data,
                accounts: vec![
                    AccountMeta::new(bridge_to_bridgers_spl.source_token_info.key(), false),
                    AccountMeta::new(bridge_to_bridgers_spl.source_token_auth.key(), true),
                    AccountMeta::new_readonly(bridge_to_bridgers_spl.token_program.key(), false),
                    AccountMeta::new(bridge_to_bridgers_spl.dest_token_info.key(), false),
                    AccountMeta::new_readonly(bridge_to_bridgers_spl.vs_info.key(), false),
                ],
            };
            invoke(
                &ix_spl,
                &bridge_to_bridgers_spl.to_account_infos()
            )?;

            Ok(BridgeResult{
                ext: "".to_string(),
            })
        }
        0x03 => {
            msg!("Processing SOL Transfer");
            let bridge_to_bridgers_sol = BridgeToBridgersSol{
                payer_account: ctx.accounts.payer.clone(),
                token_program: ctx.accounts.token_program.clone(),
                dest_token_info: ctx.remaining_accounts[1].to_account_info(),
                vs_info: ctx.remaining_accounts[2].to_account_info(),
                rent_sysvar: ctx.remaining_accounts[3].to_account_info(),
                system_program: ctx.accounts.system_program.clone(),
                mint_account_info: ctx.accounts.mint.clone(),
                pda_account_info: ctx.remaining_accounts[4].to_account_info(),
                bridgers_program: ctx.remaining_accounts[0].to_account_info(),
            };
            let ix_sol = Instruction {
                program_id: bridgers_program::id(),
                data: new_data,
                accounts: vec![
                    AccountMeta::new(bridge_to_bridgers_sol.payer_account.key(), true),
                    AccountMeta::new_readonly(bridge_to_bridgers_sol.token_program.key(), false),
                    AccountMeta::new(bridge_to_bridgers_sol.dest_token_info.key(), false),
                    AccountMeta::new_readonly(bridge_to_bridgers_sol.vs_info.key(), false),
                    AccountMeta::new_readonly(bridge_to_bridgers_sol.rent_sysvar.key(), false),
                    AccountMeta::new_readonly(bridge_to_bridgers_sol.system_program.key(), false),
                    AccountMeta::new_readonly(bridge_to_bridgers_sol.mint_account_info.key(), false),
                    AccountMeta::new(bridge_to_bridgers_sol.pda_account_info.key(), false),
                ],
            };
            invoke(
                &ix_sol,
                &bridge_to_bridgers_sol.to_account_infos()
            )?;
            
            Ok(BridgeResult{
                ext: "".to_string(),
            })
        }
        _ => {
            return Err(BridgersErrorCode::BridgersInvalidSelectorId.into());
        }
    }
}