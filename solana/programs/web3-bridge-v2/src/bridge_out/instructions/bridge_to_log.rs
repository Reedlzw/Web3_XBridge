use {
    crate::{
        bridge_out::{
            allbridge, cctp, debridgedln, meson, wormhole, wanchain, mayan_swift, bridgers,
            AdaptorID, BridgeTo, BridgeToArgs, BridgeToArgsExtData, 
            LogBridgeToVersion1, LogBridgeToVersion1Event, SwapType
        },
        common::{vec_to_hex_string, XBridgeErrorCode, wrapped_sol},
    }, anchor_lang::prelude::*, serde_json
};


pub fn bridge_to_log<'info>(
    ctx: Context<'_, '_, '_, 'info, BridgeTo<'info>>,
    data: BridgeToArgs,
) -> Result<()> {
    let mut data_clone = data.clone();
    let payer = ctx.accounts.payer.clone();
    let mint = ctx.accounts.mint.clone(); 
    let mut user_token_account_clone = ctx.accounts.user_token_account.clone();
    let from_bridge_token = mint.key().clone();
    let payer_address = payer.key().clone();
    let res;
    let before_balance: u64;
    let after_balance: u64;
    let swap_type = data.swap_type.clone();

    // @dev XBridge only support native sol to consume.
    // @dev if the mint is wsol, should be check the amount of native sol in the user_token_account.
    match swap_type {
        SwapType::BRIDGE => {
            msg!("swap_type: BRIDGE");
            // No fromswap case
            // Need to distinguish if from_bridge_token is wsol
            // If from_bridge_token is wsol, it's a Wormhole cross-chain transfer, 
            // where the user is exchanging SOL for WSOL, so SOL balance needs to be checked
            // If from_bridge_token is not wsol, then normally check the user's from_bridge_token balance before and after
            if from_bridge_token == wrapped_sol::ID {
                before_balance = payer.lamports();
                msg!("before_balance (SOL): {}", before_balance);
            } else {
                user_token_account_clone.reload()?;
                before_balance = user_token_account_clone.amount;
                msg!("before_balance (Token): {}", before_balance);
            }
        },
        SwapType::SWAPANDBRIDGE => {
            msg!("swap_type: SWAPANDBRIDGE");
            // With fromswap case
            // No need to distinguish if from_bridge_token is wsol, 
            // uniformly check the user's from_bridge_token balance before and after
            user_token_account_clone.reload()?;
            before_balance = user_token_account_clone.amount;
            msg!("before_balance (Token): {}", before_balance);
        }
    }

    match data.adaptor_id {
        AdaptorID::Meson => {
            let bridge_to_meson_args = meson::BridgeToMesonArgs::try_from_vec(&data.data).unwrap();
            res = meson::handler(ctx, data, bridge_to_meson_args).unwrap();
        }
        AdaptorID::Wormhole => {
            let bridge_to_wormhole_args = wormhole::BridgeToWromholeArgs::try_from_vec(&data.data).unwrap();
            res = wormhole::handler(ctx, data, bridge_to_wormhole_args).unwrap();
            data_clone.to_chain_id = match data_clone.to_chain_id {
                2 => 1,
                4 => 56,
                5 => 137,
                6 => 43114,
                10 => 250,
                13 => 8217,
                14 => 42220,
                16 => 1284,
                23 => 42161,
                24 => 10,
                _ => data_clone.to_chain_id,
            };
        }
        AdaptorID::Debridgedln => {
            let bridge_to_debridgedln_args = debridgedln::BridgeToDebridgedlnArgs::try_from_vec(&data.data).unwrap();
            res = debridgedln::handler(ctx, data, bridge_to_debridgedln_args).unwrap();
        }
        AdaptorID::Cctp => {
            res = cctp::handler(ctx, data).unwrap();
            data_clone.to_chain_id = match data_clone.to_chain_id {
                0 => 1,
                1 => 43114,
                2 => 10,
                3 => 42161,
                6 => 8453,
                7 => 137,
                8 => 784,
                _ => data_clone.to_chain_id,
            };
        }
        AdaptorID::Allbridge => {
            let bridge_to_allbridge_args = allbridge::BridgeToAllbridgeArgs::try_from_vec(&data.data).unwrap();
            res = allbridge::handler(ctx, data, bridge_to_allbridge_args).unwrap();
            data_clone.to_chain_id = match data_clone.to_chain_id {
                2 => 56,
                5 => 137,
                6 => 42161,
                8 => 43114,
                9 => 8453,
                11 => 42220,
                _ => data_clone.to_chain_id,
            };
            
            // The actual amount deducted for the allbridge bridge is not equal to the amount entered, due to the mechanism of the bridge
            user_token_account_clone.reload()?;
            data_clone.amount = before_balance.saturating_sub(user_token_account_clone.amount);

        }
        AdaptorID::Wanchain => {
            let bridge_to_wanchain_args = wanchain::BridgeToWanchainArgs::try_from_vec(&data.data).unwrap();
            res = wanchain::handler(ctx, data, bridge_to_wanchain_args).unwrap();
        }
        AdaptorID::MayanSwift => {
            let bridge_to_mayan_swift_args = mayan_swift::BridgeToMayanSwiftArgs::try_from_vec(&data.data).unwrap();
            res = mayan_swift::handler(ctx, data, bridge_to_mayan_swift_args).unwrap();
            data_clone.to_chain_id = match data_clone.to_chain_id {
                2 => 1,
                4 => 56,
                5 => 137,
                6 => 43114,
                23 => 42161,
                24 => 10,
                30 => 8453,
                _ => return Err(XBridgeErrorCode::InvalidToChainId.into()),
            };
        }
        AdaptorID::Bridgers => {
            let bridge_to_bridgers_args = bridgers::BridgeToBridgersArgs::try_from_vec(&data.data).unwrap();
            res = bridgers::handler(ctx, data, bridge_to_bridgers_args).unwrap();
        }
        _ => return Err(XBridgeErrorCode::InvalidAdaptorId.into()),
    }   

    match swap_type {
        SwapType::BRIDGE => {
            msg!("swap_type: BRIDGE");
            // No fromswap case
            // Need to distinguish if from_bridge_token is wsol
            // If from_bridge_token is wsol, it's a Wormhole cross-chain transfer, 
            // where the user is exchanging SOL for WSOL, so SOL balance needs to be checked
            // If from_bridge_token is not wsol, then normally check the user's from_bridge_token balance before and after
            if from_bridge_token == wrapped_sol::ID {
                after_balance = payer.lamports();
                msg!("before_balance (SOL): {}", before_balance);
                msg!("after_balance (SOL): {}", after_balance);
                msg!("data_clone.amount: {}", data_clone.amount);
                require!(
                    before_balance >= after_balance + data_clone.amount,
                    XBridgeErrorCode::AmountMustEqualConsumed
                );
            } else {
                user_token_account_clone.reload()?;
                after_balance = user_token_account_clone.amount;
                msg!("before_balance (Token): {}", before_balance);
                msg!("after_balance (Token): {}", after_balance);
                msg!("data_clone.amount: {}", data_clone.amount);
                require!(
                    before_balance == after_balance + data_clone.amount,
                    XBridgeErrorCode::AmountMustEqualConsumed
                );
            }
        },
        SwapType::SWAPANDBRIDGE => {
            msg!("swap_type: SWAPANDBRIDGE");
            // With fromswap case
            // No need to distinguish if from_bridge_token is wsol, 
            // uniformly check the user's from_bridge_token balance before and after
            user_token_account_clone.reload()?;
            after_balance = user_token_account_clone.amount;
            msg!("before_balance (Token): {}", before_balance);
            msg!("after_balance (Token): {}", after_balance);
            msg!("data_clone.amount: {}", data_clone.amount);
            if data_clone.adaptor_id == AdaptorID::Wormhole && mint.key() == wrapped_sol::ID {
                // Special handling is required because Wormhole rounds down decimal places when transferring WSOL across chains
                let adjusted_amount = data_clone.amount
                    .checked_div(10)
                    .ok_or(XBridgeErrorCode::CalculationError)?
                    .checked_mul(10)
                    .ok_or(XBridgeErrorCode::CalculationError)?;
                require!(
                    before_balance == after_balance + adjusted_amount,
                    XBridgeErrorCode::AmountMustEqualConsumed
                );
            } else {
                require!(
                    before_balance == after_balance + data_clone.amount,
                    XBridgeErrorCode::AmountMustEqualConsumed
                );
            }
        }
    }

    let ext_data: BridgeToArgsExtData = BridgeToArgsExtData::try_from_slice(&data_clone.ext_data).unwrap();
    let user_address_str = match String::from_utf8(ext_data.user_address.clone()) {
        Ok(user_address_str) => user_address_str,
        Err(e) => {
            msg!("Failed to convert user_address to string: {:?}", e);
            return Err(XBridgeErrorCode::InvalidUserAddress.into());
        }
    };
    let to_hex_string: String = vec_to_hex_string(data_clone.to);

    emit!(LogBridgeToVersion1Event {
        order_id: data_clone.order_id.to_string(),
        adaptor_id: data_clone.adaptor_id as u8,
        to: to_hex_string.clone(),
        amount: data_clone.amount,
        swap_type: data_clone.swap_type as u8,
        to_chain_id: data_clone.to_chain_id as u64,
        bridge_token: from_bridge_token.to_string(),
        src_chain_id: 501,
        from: payer_address.to_string(),
        user_address: user_address_str.clone(),
        ext:res.ext.clone()
    });

    let xbridge_log: LogBridgeToVersion1 = LogBridgeToVersion1 {
        order_id: data_clone.order_id.to_string(),
        adaptor_id: data_clone.adaptor_id as u8,
        to: to_hex_string.clone(),
        amount: data_clone.amount,
        swap_type: data_clone.swap_type as u8,
        to_chain_id: data_clone.to_chain_id as u64,
        bridge_token: from_bridge_token.to_string(),
        src_chain_id: 501,
        from: payer_address.to_string(),
        user_address: user_address_str.clone(),
        ext:res.ext.clone()
    };
    msg!("LogBridgeToVersion1:{}", serde_json::to_string(&xbridge_log).unwrap());

    Ok(())
}
