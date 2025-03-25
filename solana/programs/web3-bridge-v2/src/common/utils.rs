use {
    tiny_keccak::{Hasher, Keccak},
    hex::encode,
    anchor_lang::prelude::*,
    crate::common::XBridgeErrorCode,
};

pub fn public_key_to_address(public_key: &[u8]) -> String {
    let mut hasher = Keccak::v256();
    let mut hash = [0u8; 32];
    hasher.update(public_key);
    hasher.finalize(&mut hash);
    
    let address_bytes = &hash[12..];
    let address = encode(address_bytes);
    format!("0x{}", address)
}

pub fn vec_to_hex_string(vec: Vec<u8>) -> String {
    vec.iter().map(|byte| format!("{:02x}", byte)).collect()
}

pub fn safe_to_u16(value: u64) -> Result<u16> {
    require!(value <= u16::MAX as u64, XBridgeErrorCode::UnSafeCovert);
    Ok(value as u16)
}

pub fn safe_to_fixed_bytes<const N: usize>(data: Vec<u8>) -> Result<[u8; N]> {
    require!(data.len() <= N, XBridgeErrorCode::UnSafeCovert);
    let mut fixed = [0u8; N];
    let len = data.len().min(N);
    fixed[..len].copy_from_slice(&data[..len]);
    Ok(fixed)
}
