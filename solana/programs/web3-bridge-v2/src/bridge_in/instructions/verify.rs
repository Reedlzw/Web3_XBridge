use {
    crate::{
        bridge_in::{
            ContractConfig,
            BridgeMessage,
            ToSwapMessageState,
        },
        common::{
            public_key_to_address,
            XBridgeErrorCode,
            TEST_MPC_STR,
            TEST_MPC_STR_2
        },
    },
    anchor_lang::{
        prelude::*,
        solana_program::{
            secp256k1_recover, system_program,
        },
    },
    tiny_keccak::{Hasher, Keccak},
};

#[derive(Accounts)]
#[instruction(params: VerifyArgs)] 
pub struct Verify<'info> {
    #[account(mut)]
    /// Payer.
    /// This account is user to pay for the transaction and bridge token.
    /// CHECK: This account is user to pay for the transaction and bridge token.
    pub payer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + ToSwapMessageState::INIT_SPACE,
        seeds = [
            b"toswap_message",
            &BridgeMessage::from_message(&params.message)?.src_chain_id()[..],
            &BridgeMessage::from_message(&params.message)?.src_tx_hash()[..]
        ],
        bump
        
    )]
    /// CHECK: 
    pub toswap_message_request: Box<Account<'info, ToSwapMessageState>>,

    #[account(
        constraint = payer.key() == contract_config.mpc ||
                    payer.key().to_string() == TEST_MPC_STR ||
                    payer.key().to_string() == TEST_MPC_STR_2 @ XBridgeErrorCode::Unauthorized,
        constraint = !contract_config.paused @ XBridgeErrorCode::AlreadyPaused,
        seeds = [b"contract_config"],
        bump
    )]
    /// CHECK:
    pub contract_config: Account<'info, ContractConfig>,

    /// System Program.
    /// "11111111111111111111111111111111"
    /// CHECK: fixed
    #[account(address = system_program::id())]
    pub system_program: Program<'info, System>,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct VerifyArgs {
    pub message: Vec<u8>,
    pub signature: Vec<u8>,
    pub orderid: u128,
}

pub fn verify<'info>(
    _ctx: Context<'_, '_, '_, 'info, Verify<'info>>,
    data: VerifyArgs,
) -> Result<()> {
    let contract_config = &mut _ctx.accounts.contract_config;

    // use the message and signature to recover the signer and verify if the message matches signed by the oracle.
    let raw_message = data.message.clone();
    let mut hasher = Keccak::v256();
    let mut raw_message_hash = [0u8; 32];
    hasher.update(&raw_message);
    hasher.finalize(&mut raw_message_hash);
    let prefix = b"\x19Ethereum Signed Message:\n32";
    let mut hasher = Keccak::v256();
    let mut prefixed_message_hash = [0u8; 32];
    hasher.update(prefix);
    hasher.update(&raw_message_hash);
    hasher.finalize(&mut prefixed_message_hash);
    let message_hash_ref: &[u8] = &prefixed_message_hash;
    let raw_signature = data.signature;
    let r = &raw_signature[0..32];
    let s = &raw_signature[32..64];
    let v = raw_signature[95];
    let mut signature_bytes = [0u8; 64];
    signature_bytes[..32].copy_from_slice(r);
    signature_bytes[32..].copy_from_slice(s);
    let signature = libsecp256k1::Signature::parse_standard_slice(&signature_bytes)
        .map_err(|_| XBridgeErrorCode::InvalidSignature)?;
    if signature.s.is_high() {
        return Err(XBridgeErrorCode::InvalidSignature.into());
    }
    let recovery_id = if v >= 27 { v - 27 } else { v };
    let recovery_pubkey = secp256k1_recover::secp256k1_recover(
        message_hash_ref,
        recovery_id.into(),
        &signature_bytes,
    ).expect("Public key recovery failed");
    let recovered_pubkey_serialized: Vec<u8> = recovery_pubkey.0.to_vec();
    let eth_address = public_key_to_address(&recovered_pubkey_serialized);
    msg!("Recovered Ethereum address: {:?}", eth_address);
    let eth_address_bytes = hex::decode(&eth_address[2..])
        .map_err(|_| XBridgeErrorCode::DeserializationError)?;
    let eth_address_array: [u8; 20] = eth_address_bytes
        .try_into()
        .map_err(|_| XBridgeErrorCode::DeserializationError)?;
    require!(
        eth_address_array == contract_config.oracle,
        XBridgeErrorCode::NotOracleProxy
    );

    // use src_chain_id and src_tx_hash as seeds to create a PDA account for each transaction from the source chain.
    let oracle_src_chain_message = BridgeMessage::try_from_slice(&data.message)?;
    BridgeMessage::msg_oracle_data(&oracle_src_chain_message, data.orderid);

    if _ctx.accounts.toswap_message_request.data.iter().all(|&x| x == 0) {
        // if toswap_message_request is created for the first time
        // write message to it
        let mut fixed_data: [u8; 160] = [0; 160];
        let message_len = data.message.len();
        if message_len > 160 {
            return Err(XBridgeErrorCode::DataTooLong.into());
        }
        fixed_data[..message_len].copy_from_slice(&data.message[..message_len]);

        let toswap_message_state_data = &mut _ctx.accounts.toswap_message_request;
        toswap_message_state_data.is_used = false;
        toswap_message_state_data.authority = _ctx.accounts.payer.key();
        toswap_message_state_data.authority_program = *_ctx.program_id;
        toswap_message_state_data.data = fixed_data;
        msg!("toswap_message_state_data: {:?}", toswap_message_state_data);
    } else {
        // if the account has been created
        // when the toswap_message_request.is_used != true (no dex or refund has been performed)
        // the signature verification has passed
        // allowed to rewrite information to the toswap_message_request account
        let toswap_message_state_data = &mut _ctx.accounts.toswap_message_request;
        require!(
            !toswap_message_state_data.is_used,
            XBridgeErrorCode::ToswapAlreadyUsed
        );

        let mut fixed_data: [u8; 160] = [0; 160];
        let message_len = data.message.len();
        if message_len > 160 {
            return Err(XBridgeErrorCode::DataTooLong.into());
        }
        fixed_data[..message_len].copy_from_slice(&data.message[..message_len]);
        toswap_message_state_data.data = fixed_data;

        msg!("Updated toswap_message_state_data: {:?}", toswap_message_state_data);
    }

    Ok(())
}