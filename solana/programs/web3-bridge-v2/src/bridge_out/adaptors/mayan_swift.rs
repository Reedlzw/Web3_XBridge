use {
    crate::{
        bridge_out::{BridgeResult, BridgeTo, BridgeToArgs},
        common::{
            mayan_fee_manager_program as MayanFeeManagerProgram,
            mayan_swift_program as MayanSwiftProgram, safe_to_fixed_bytes, safe_to_u16,
        },
    },
    anchor_lang::{
        prelude::*,
        solana_program::{instruction::Instruction, program::invoke},
    },
    anchor_spl::{
        associated_token::{self, Create},
        token::{self, Mint, Token, TokenAccount, Transfer},
    },
};

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, BridgeTo<'info>>,
    data: BridgeToArgs,
    bridge_to_mayan_args: BridgeToMayanSwiftArgs,
) -> Result<BridgeResult> {

    // msg!("order_hash: {}", vec_to_hex_string(bridge_to_mayan_args.order_hash.to_vec()));
    associated_token::create_idempotent(
        CpiContext::new(
            ctx.accounts.associated_token_program.to_account_info(),
            Create {
                payer: ctx.accounts.payer.to_account_info(),
                associated_token: ctx.remaining_accounts[1].to_account_info(),
                authority: ctx.remaining_accounts[0].to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
            },
        )
    )?;
    
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_account.to_account_info(),
                to: ctx.remaining_accounts[1].to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        ),
        data.amount,
    )?;
    

    let mayan_swift_args = MayanSwiftArgs {
        instruction: MayanSwiftArgs::INSTRUCTION_BYTES,
        amount_in_min: data.amount,
        native_input: bridge_to_mayan_args.native_input as u8,
        fee_submit: bridge_to_mayan_args.fee_submit,
        dest_address: safe_to_fixed_bytes::<32>(data.to)?,
        destination_chain: safe_to_u16(data.to_chain_id)?,
        token_out: bridge_to_mayan_args.token_out,
        amount_out_min: bridge_to_mayan_args.amount_out_min,
        gas_drop: 0,
        fee_cancel: bridge_to_mayan_args.fee_cancel,
        fee_refund: bridge_to_mayan_args.fee_refund,
        deadline: bridge_to_mayan_args.deadline,
        ref_address: [0; 32],
        fee_rate_ref: 0,
        fee_rate_mayan: bridge_to_mayan_args.fee_rate_mayan,
        auction_mode: bridge_to_mayan_args.auction_mode,
        random_key: bridge_to_mayan_args.random_key,
    };
    let mayan_swift_data_bytes = MayanSwiftArgs::encode(&mayan_swift_args);

    let bridge_to_mayan = BridgeToMayan {
        trader: ctx.accounts.payer.clone(),
        relayer: ctx.accounts.payer.clone(),
        state: ctx.remaining_accounts[0].to_account_info(),
        state_account: ctx.remaining_accounts[1].to_account_info(),
        relayer_account: ctx.accounts.user_token_account.clone(),
        mint: ctx.accounts.mint.clone(),
        fee_manager_program: ctx.remaining_accounts[3].to_account_info(),
        token_program: ctx.accounts.token_program.clone(),
        system_program: ctx.accounts.system_program.clone(),
        swift_program: ctx.remaining_accounts[2].to_account_info(),
    };

    let ix = Instruction {
        program_id: MayanSwiftProgram::id(),
        data: mayan_swift_data_bytes,
        accounts: vec![
            AccountMeta::new(bridge_to_mayan.trader.key(), true),
            AccountMeta::new(bridge_to_mayan.relayer.key(), true),
            AccountMeta::new(bridge_to_mayan.state.key(), false),
            AccountMeta::new(bridge_to_mayan.state_account.key(), false),
            AccountMeta::new(bridge_to_mayan.relayer_account.key(), false),
            AccountMeta::new_readonly(bridge_to_mayan.mint.key(), false),
            AccountMeta::new_readonly(bridge_to_mayan.fee_manager_program.key(), false),
            AccountMeta::new_readonly(bridge_to_mayan.token_program.key(), false),
            AccountMeta::new_readonly(bridge_to_mayan.system_program.key(), false),
        ],
    };

    invoke(
        &ix, 
        &bridge_to_mayan.to_account_infos()
    )?;

    Ok(BridgeResult {
        ext: "".to_string(),
    })
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct BridgeToMayanSwiftArgs {
    pub native_input: bool,
    pub fee_submit: u64,
    pub token_out: [u8; 32],
    pub amount_out_min: u64,
    pub fee_cancel: u64,
    pub fee_refund: u64,
    pub deadline: u64,
    pub fee_rate_mayan: u8,
    pub auction_mode: u8,
    pub random_key: [u8; 32],
    pub order_hash: [u8; 32],
}

impl BridgeToMayanSwiftArgs {
    pub fn try_from_vec(data: &[u8]) -> Result<BridgeToMayanSwiftArgs> {
        let decoded_args: BridgeToMayanSwiftArgs = BridgeToMayanSwiftArgs::try_from_slice(&data)?;
        Ok(decoded_args)
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MayanSwiftArgs {
    pub instruction: [u8; 8],   // 8 bytes
    pub amount_in_min: u64,     // 8 bytes
    pub native_input: u8,       // 1 byte
    pub fee_submit: u64,        // 8 bytes
    pub dest_address: [u8; 32], // 32 bytes
    pub destination_chain: u16, // 2 bytes
    pub token_out: [u8; 32],    // 32 bytes
    pub amount_out_min: u64,    // 8 bytes
    pub gas_drop: u64,          // 8 bytes
    pub fee_cancel: u64,        // 8 bytes
    pub fee_refund: u64,        // 8 bytes
    pub deadline: u64,          // 8 bytes
    pub ref_address: [u8; 32],  // 32 bytes
    pub fee_rate_ref: u8,       // 1 byte
    pub fee_rate_mayan: u8,     // 1 byte
    pub auction_mode: u8,       // 1 byte
    pub random_key: [u8; 32],   // 32 bytes
}

impl MayanSwiftArgs {
    pub const INSTRUCTION_BYTES: [u8; 8] = [32, 76, 41, 12, 39, 162, 132, 219];
    
    pub fn encode(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(239); // Total size = 239 bytes

        data.extend_from_slice(&self.instruction);
        data.extend_from_slice(&self.amount_in_min.to_le_bytes());
        data.push(self.native_input);
        data.extend_from_slice(&self.fee_submit.to_le_bytes());
        data.extend_from_slice(&self.dest_address);
        data.extend_from_slice(&self.destination_chain.to_le_bytes());
        data.extend_from_slice(&self.token_out);
        data.extend_from_slice(&self.amount_out_min.to_le_bytes());
        data.extend_from_slice(&self.gas_drop.to_le_bytes());
        data.extend_from_slice(&self.fee_cancel.to_le_bytes());
        data.extend_from_slice(&self.fee_refund.to_le_bytes());
        data.extend_from_slice(&self.deadline.to_le_bytes());
        data.extend_from_slice(&self.ref_address);
        data.push(self.fee_rate_ref);
        data.push(self.fee_rate_mayan);
        data.push(self.auction_mode);
        data.extend_from_slice(&self.random_key);

        data
    }

    pub fn new(
        amount_in_min: u64,
        native_input: u8,
        fee_submit: u64,
        dest_address: [u8; 32],
        destination_chain: u16,
        token_out: [u8; 32],
        amount_out_min: u64,
        fee_rate_mayan: u8,
        auction_mode: u8,
        random_key: [u8; 32],
    ) -> Self {
        Self {
            instruction: Self::INSTRUCTION_BYTES,
            amount_in_min,
            native_input,
            fee_submit,
            dest_address,
            destination_chain,
            token_out,
            amount_out_min,
            gas_drop: 0,
            fee_cancel: 0,
            fee_refund: 0,
            deadline: 0,
            ref_address: [0; 32],
            fee_rate_ref: 0,
            fee_rate_mayan,
            auction_mode,
            random_key,
        }
    }
}

#[derive(Accounts)]
#[instruction(bridge_to_mayan_args: BridgeToMayanSwiftArgs, bridge_to_args: BridgeToArgs)]
pub struct BridgeToMayan<'info> {
    #[account(mut)]
    /// Trader.
    /// This account is used to pay for the bridge token.
    /// CHECK: This account is used to pay for the bridge token.
    pub trader: Signer<'info>,

    #[account(mut)]
    /// Relayer.
    /// This account is used to pay for the transaction as payer.
    /// CHECK: This account is used to pay for the transaction as payer.
    pub relayer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"STATE_SOURCE".as_ref(), &bridge_to_mayan_args.order_hash],
        bump,
        seeds::program = MayanSwiftProgram::id(),
    )]
    /// CHECK: This account is used to create ATA account to receive token for mayan.
    pub state: AccountInfo<'info>,

    #[account(
        mut,
        // associated_token::mint = mint,
        // associated_token::authority = state,
    )]
    /// State Token Account
    /// ATA: mint = mint, owner = relayer,
    /// CHECK: This Account is used to receive token for mayan,
    pub state_account: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = relayer,
    )]
    /// User Token Account
    /// ATA: mint = mint, owner = payer,
    /// CHECK: This Account is used to send token for user,
    pub relayer_account: Account<'info, TokenAccount>,

    /// CHECK: Mint (read-only).
    pub mint: Account<'info, Mint>,

    /// Mayan Fee Manager Program
    /// "5VtQHnhs2pfVEr68qQsbTRwKh4JV5GTu9mBHgHFxpHeQ"
    /// CHECK: Mayan Fee Manager Program ID
    #[account(address = MayanFeeManagerProgram::id())]
    pub fee_manager_program: AccountInfo<'info>,

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

    /// Mayan Swift Program
    /// "BLZRi6frs4X4DNLw56V4EXai1b6QVESN1BhHBTYM9VcY"
    /// CHECK: Mayan Swift Program ID
    #[account(address = MayanSwiftProgram::id())]
    pub swift_program: AccountInfo<'info>,
}
