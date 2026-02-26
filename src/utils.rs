//! Utility functions for parsing and conversion.

use crate::constants::tokens;
use crate::types::TradeType;
use solana_sdk::pubkey::Pubkey;

pub fn decode_instruction_data(data: &[u8]) -> Vec<u8> {
    data.to_vec()
}

pub fn decode_instruction_data_base58(data: &str) -> Result<Vec<u8>, bs58::decode::Error> {
    bs58::decode(data).into_vec()
}

pub fn get_pubkey_string(value: &[u8; 32]) -> String {
    bs58::encode(value).into_string()
}

pub fn pubkey_to_string(pubkey: &Pubkey) -> String {
    pubkey.to_string()
}

pub fn convert_to_ui_amount(amount: u64, decimals: u8) -> f64 {
    if decimals == 0 {
        return amount as f64;
    }
    amount as f64 / 10_f64.powi(decimals as i32)
}

pub fn convert_to_ui_amount_u128(amount: u128, decimals: u8) -> f64 {
    if decimals == 0 {
        return amount as f64;
    }
    amount as f64 / 10_f64.powi(decimals as i32)
}

pub fn get_trade_type(in_mint: &str, out_mint: &str) -> TradeType {
    if in_mint == tokens::SOL {
        return TradeType::Buy;
    }
    if out_mint == tokens::SOL {
        return TradeType::Sell;
    }
    if is_known_stable(in_mint) {
        return TradeType::Buy;
    }
    TradeType::Sell
}

fn is_known_stable(mint: &str) -> bool {
    matches!(
        mint,
        tokens::USDC | tokens::USDT | tokens::USD1 | tokens::USDG | tokens::PYUSD
            | tokens::EURC | tokens::USDY | tokens::FDUSD
    )
}

pub fn get_transfer_token_mint(token1: Option<&str>, token2: Option<&str>) -> Option<String> {
    match (token1, token2) {
        (Some(a), Some(b)) if a == b => Some(a.to_string()),
        (Some(a), _) if a != tokens::SOL => Some(a.to_string()),
        (_, Some(b)) if b != tokens::SOL => Some(b.to_string()),
        (a, b) => a.or(b).map(String::from),
    }
}

#[derive(Clone)]
pub struct IdxSortable<T> {
    pub item: T,
    pub idx: String,
}

pub fn sort_by_idx<T: Clone>(items: &[(T, String)]) -> Vec<T> {
    if items.len() <= 1 {
        return items.iter().map(|(t, _)| t.clone()).collect();
    }
    let mut with_idx: Vec<_> = items
        .iter()
        .map(|(t, idx)| (t.clone(), idx.clone()))
        .collect();
    with_idx.sort_by(|a, b| {
        let pa: Vec<&str> = a.1.split('-').collect();
        let pb: Vec<&str> = b.1.split('-').collect();
        let a_main: u32 = pa.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
        let a_sub: u32 = pa.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let b_main: u32 = pb.get(0).and_then(|s| s.parse().ok()).unwrap_or(0);
        let b_sub: u32 = pb.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        a_main.cmp(&b_main).then(a_sub.cmp(&b_sub))
    });
    with_idx.into_iter().map(|(t, _)| t).collect()
}

pub fn get_final_swap(
    trades: &[crate::types::TradeInfo],
    dex_amm: Option<&str>,
    dex_route: Option<&str>,
) -> Option<crate::types::TradeInfo> {
    if trades.is_empty() {
        return None;
    }
    if trades.len() == 1 {
        return Some(trades[0].clone());
    }
    let sorted = {
        let with_idx: Vec<_> = trades.iter().map(|t| (t.clone(), t.idx.clone())).collect();
        sort_by_idx(&with_idx)
    };
    let input_trade = sorted.first()?;
    let output_trade = sorted.last()?;
    let mut pools: Vec<String> = Vec::new();
    let mut input_amount: u128 = 0;
    let mut output_amount: u128 = 0;
    for trade in &sorted {
        if trade.input_token.mint == input_trade.input_token.mint {
            input_amount += trade.input_token.amount_raw.parse::<u128>().unwrap_or(0);
        }
        if trade.output_token.mint == output_trade.output_token.mint {
            output_amount += trade.output_token.amount_raw.parse::<u128>().unwrap_or(0);
        }
        if let Some(p) = trade.pool.first() {
            if !pools.contains(p) {
                pools.push(p.clone());
            }
        }
    }
    Some(crate::types::TradeInfo {
        user: input_trade.user.clone(),
        trade_type: get_trade_type(&input_trade.input_token.mint, &output_trade.output_token.mint),
        pool: pools,
        input_token: crate::types::TokenInfo {
            mint: input_trade.input_token.mint.clone(),
            amount: crate::utils::convert_to_ui_amount_u128(
                input_amount,
                input_trade.input_token.decimals,
            ),
            amount_raw: input_amount.to_string(),
            decimals: input_trade.input_token.decimals,
            authority: input_trade.input_token.authority.clone(),
            destination: input_trade.input_token.destination.clone(),
            destination_owner: input_trade.input_token.destination_owner.clone(),
            source: input_trade.input_token.source.clone(),
        },
        output_token: crate::types::TokenInfo {
            mint: output_trade.output_token.mint.clone(),
            amount: crate::utils::convert_to_ui_amount_u128(
                output_amount,
                output_trade.output_token.decimals,
            ),
            amount_raw: output_amount.to_string(),
            decimals: output_trade.output_token.decimals,
            authority: output_trade.output_token.authority.clone(),
            destination: output_trade.output_token.destination.clone(),
            destination_owner: output_trade.output_token.destination_owner.clone(),
            source: output_trade.output_token.source.clone(),
        },
        slippage_bps: input_trade.slippage_bps,
        fee: input_trade.fee.clone(),
        fees: input_trade.fees.clone(),
        program_id: input_trade.program_id.clone(),
        amm: dex_amm.map(String::from).or_else(|| input_trade.amm.clone()),
        amms: input_trade.amms.clone(),
        route: dex_route.map(String::from).or_else(|| input_trade.route.clone()),
        slot: input_trade.slot,
        timestamp: input_trade.timestamp,
        signature: input_trade.signature.clone(),
        idx: input_trade.idx.clone(),
        signer: input_trade.signer.clone(),
    })
}
