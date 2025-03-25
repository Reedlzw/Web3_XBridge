use {
    anchor_lang::prelude::*,
    serde::{Deserialize, Serialize},
};

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct BridgeMessage {
    pub src_chain_id: [u8; 32],
    pub src_tx_hash: [u8; 32],
    pub to: [u8; 32],
    pub from_token: [u8; 32],
    pub from_amount: [u8; 32],      
}
impl BridgeMessage {
    pub fn from_message(message: &[u8]) -> Result<Self> {
        BridgeMessage::try_from_slice(message).map_err(|e| e.into())
    }

    pub fn src_chain_id(&self) -> &[u8; 32] {
        &self.src_chain_id
    }

    pub fn src_tx_hash(&self) -> &[u8; 32] {
        &self.src_tx_hash
    }

    pub fn msg_oracle_data(
        parsed_data: &BridgeMessage,
        orderid: u128,
    ) {
        let src_chain_id = u128::from_be_bytes(parsed_data.src_chain_id[16..32].try_into().expect("slice with incorrect length"));
        let src_tx_hash = hex::encode(parsed_data.src_tx_hash);
        let to_base58 = bs58::encode(parsed_data.to).into_string();
        let from_token_base58 = bs58::encode(parsed_data.from_token).into_string();
        let from_amount = u64::from_be_bytes(parsed_data.from_amount[24..32].try_into().expect("slice with incorrect length"));
    
        let oracle_data_log: BridgeMessageLog = BridgeMessageLog {
            src_chain_id: src_chain_id,
            src_tx_hash: src_tx_hash,
            to: to_base58,
            from_token: from_token_base58,
            from_amount: from_amount,
            orderid: orderid
        };
        msg!("Parsed oracle data log:{}", serde_json::to_string(&oracle_data_log).unwrap())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Serialize, Deserialize)]
struct BridgeMessageLog {
    pub src_chain_id: u128,
    pub src_tx_hash: String,
    pub to: String,               
    pub from_token: String,
    pub from_amount: u64,  
    pub orderid: u128,    
}