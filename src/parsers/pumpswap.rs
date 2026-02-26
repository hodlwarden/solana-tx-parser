//! Pumpswap DEX parser - event-based (buy/sell event discriminators).

use crate::binary_reader::BinaryReader;
use crate::constants::{dex_programs, discriminators};
use crate::transaction_adapter::TransactionAdapter;
use crate::types::{ClassifiedInstruction, DexInfo, TradeInfo, TransferData};
use crate::utils::{convert_to_ui_amount, get_trade_type};
use std::collections::HashMap;

pub struct PumpswapParser<'a> {
    adapter: &'a TransactionAdapter<'a>,
    dex_info: DexInfo,
    _transfer_actions: HashMap<String, Vec<TransferData>>,
    classified_instructions: Vec<ClassifiedInstruction>,
}

impl<'a> PumpswapParser<'a> {
    pub fn new(
        adapter: &'a TransactionAdapter<'a>,
        dex_info: DexInfo,
        transfer_actions: HashMap<String, Vec<TransferData>>,
        classified_instructions: Vec<ClassifiedInstruction>,
    ) -> Self {
        Self {
            adapter,
            dex_info,
            _transfer_actions: transfer_actions,
            classified_instructions,
        }
    }

    pub fn process_trades(&self) -> Vec<TradeInfo> {
        let mut trades = Vec::new();
        for ci in &self.classified_instructions {
            if ci.program_id != dex_programs::PUMP_SWAP.id {
                continue;
            }
            let data = &ci.instruction.data;
            if data.len() < 16 {
                continue;
            }
            let idx = format!("{}-{}", ci.outer_index, ci.inner_index.unwrap_or(0));
            let event_data = &data[16..];
            if data[0..16] == discriminators::PUMPSWAP_BUY_EVENT {
                if let Some(evt) = decode_buy_event(event_data) {
                    let input_mint = self.adapter.spl_token_map.get(&evt.user_quote_token_account).map(|t| t.mint.clone());
                    let output_mint = self.adapter.spl_token_map.get(&evt.user_base_token_account).map(|t| t.mint.clone());
                    let fee_mint = self.adapter.spl_token_map.get(&evt.protocol_fee_recipient_token_account).map(|t| t.mint.clone());
                    if let (Some(in_m), Some(out_m), Some(fee_m)) = (input_mint, output_mint, fee_mint) {
                        let in_dec = self.adapter.get_token_decimals(&in_m);
                        let out_dec = self.adapter.get_token_decimals(&out_m);
                        let fee_dec = self.adapter.get_token_decimals(&fee_m);
                        let fee_amt = evt.protocol_fee + evt.coin_creator_fee;
                        let trade = TradeInfo {
                            user: evt.user.clone(),
                            trade_type: get_trade_type(&in_m, &out_m),
                            pool: vec![evt.pool.clone()],
                            input_token: crate::types::TokenInfo {
                                mint: in_m.clone(),
                                amount: convert_to_ui_amount(evt.quote_amount_in_with_lp_fee, in_dec),
                                amount_raw: evt.quote_amount_in_with_lp_fee.to_string(),
                                decimals: in_dec,
                                authority: None,
                                destination: None,
                                destination_owner: None,
                                source: None,
                            },
                            output_token: crate::types::TokenInfo {
                                mint: out_m.clone(),
                                amount: convert_to_ui_amount(evt.base_amount_out, out_dec),
                                amount_raw: evt.base_amount_out.to_string(),
                                decimals: out_dec,
                                authority: None,
                                destination: None,
                                destination_owner: None,
                                source: None,
                            },
                            slippage_bps: None,
                            fee: Some(crate::types::FeeInfo {
                                mint: fee_m.clone(),
                                amount: convert_to_ui_amount(fee_amt, fee_dec),
                                amount_raw: fee_amt.to_string(),
                                decimals: fee_dec,
                                dex: Some(dex_programs::PUMP_SWAP.name.to_string()),
                                type_: Some("protocol".to_string()),
                                recipient: Some(evt.protocol_fee_recipient.clone()),
                            }),
                            fees: None,
                            program_id: Some(dex_programs::PUMP_SWAP.id.to_string()),
                            amm: Some(dex_programs::PUMP_SWAP.name.to_string()),
                            amms: None,
                            route: self.dex_info.route.clone(),
                            slot: self.adapter.slot(),
                            timestamp: self.adapter.block_time(),
                            signature: self.adapter.signature(),
                            idx: idx.clone(),
                            signer: Some(self.adapter.signers()),
                        };
                        trades.push(trade);
                    }
                }
            } else if data[0..16] == discriminators::PUMPSWAP_SELL_EVENT {
                if let Some(evt) = decode_sell_event(event_data) {
                    let input_mint = self.adapter.spl_token_map.get(&evt.user_base_token_account).map(|t| t.mint.clone());
                    let output_mint = self.adapter.spl_token_map.get(&evt.user_quote_token_account).map(|t| t.mint.clone());
                    let fee_mint = self.adapter.spl_token_map.get(&evt.protocol_fee_recipient_token_account).map(|t| t.mint.clone());
                    if let (Some(in_m), Some(out_m), Some(fee_m)) = (input_mint, output_mint, fee_mint) {
                        let in_dec = self.adapter.get_token_decimals(&in_m);
                        let out_dec = self.adapter.get_token_decimals(&out_m);
                        let fee_dec = self.adapter.get_token_decimals(&fee_m);
                        let fee_amt = evt.protocol_fee + evt.coin_creator_fee;
                        let trade = TradeInfo {
                            user: evt.user.clone(),
                            trade_type: get_trade_type(&in_m, &out_m),
                            pool: vec![evt.pool.clone()],
                            input_token: crate::types::TokenInfo {
                                mint: in_m.clone(),
                                amount: convert_to_ui_amount(evt.base_amount_in, in_dec),
                                amount_raw: evt.base_amount_in.to_string(),
                                decimals: in_dec,
                                authority: None,
                                destination: None,
                                destination_owner: None,
                                source: None,
                            },
                            output_token: crate::types::TokenInfo {
                                mint: out_m.clone(),
                                amount: convert_to_ui_amount(evt.user_quote_amount_out, out_dec),
                                amount_raw: evt.user_quote_amount_out.to_string(),
                                decimals: out_dec,
                                authority: None,
                                destination: None,
                                destination_owner: None,
                                source: None,
                            },
                            slippage_bps: None,
                            fee: Some(crate::types::FeeInfo {
                                mint: fee_m.clone(),
                                amount: convert_to_ui_amount(fee_amt, fee_dec),
                                amount_raw: evt.protocol_fee.to_string(),
                                decimals: fee_dec,
                                dex: Some(dex_programs::PUMP_SWAP.name.to_string()),
                                type_: None,
                                recipient: Some(evt.protocol_fee_recipient.clone()),
                            }),
                            fees: None,
                            program_id: Some(dex_programs::PUMP_SWAP.id.to_string()),
                            amm: Some(dex_programs::PUMP_SWAP.name.to_string()),
                            amms: None,
                            route: self.dex_info.route.clone(),
                            slot: self.adapter.slot(),
                            timestamp: self.adapter.block_time(),
                            signature: self.adapter.signature(),
                            idx: idx.clone(),
                            signer: Some(self.adapter.signers()),
                        };
                        trades.push(trade);
                    }
                }
            }
        }
        trades
    }
}

struct PumpswapBuyEvent {
    base_amount_out: u64,
    quote_amount_in_with_lp_fee: u64,
    protocol_fee: u64,
    coin_creator_fee: u64,
    pool: String,
    user: String,
    user_base_token_account: String,
    user_quote_token_account: String,
    protocol_fee_recipient: String,
    protocol_fee_recipient_token_account: String,
}

struct PumpswapSellEvent {
    base_amount_in: u64,
    user_quote_amount_out: u64,
    protocol_fee: u64,
    coin_creator_fee: u64,
    pool: String,
    user: String,
    user_base_token_account: String,
    user_quote_token_account: String,
    protocol_fee_recipient: String,
    protocol_fee_recipient_token_account: String,
}

fn decode_buy_event(data: &[u8]) -> Option<PumpswapBuyEvent> {
    let mut r = BinaryReader::new(data);
    let _timestamp = r.read_i64_le().ok()?;
    let base_amount_out = r.read_u64_le().ok()?;
    let _max_quote = r.read_u64_le().ok()?;
    let _ub = r.read_u64_le().ok()?;
    let _uq = r.read_u64_le().ok()?;
    let _pb = r.read_u64_le().ok()?;
    let _pq = r.read_u64_le().ok()?;
    let _quote_in = r.read_u64_le().ok()?;
    let _lp_bps = r.read_u64_le().ok()?;
    let _lp_fee = r.read_u64_le().ok()?;
    let _proto_bps = r.read_u64_le().ok()?;
    let protocol_fee = r.read_u64_le().ok()?;
    let quote_amount_in_with_lp_fee = r.read_u64_le().ok()?;
    let _user_quote_in = r.read_u64_le().ok()?;
    let pool = r.read_pubkey().ok()?;
    let user = r.read_pubkey().ok()?;
    let user_base_token_account = r.read_pubkey().ok()?;
    let user_quote_token_account = r.read_pubkey().ok()?;
    let protocol_fee_recipient = r.read_pubkey().ok()?;
    let protocol_fee_recipient_token_account = r.read_pubkey().ok()?;
    let (coin_creator_fee, _) = if r.remaining() >= 48 {
        let _creator = r.read_pubkey().ok()?;
        let _bps = r.read_u64_le().ok()?;
        (r.read_u64_le().ok()?, ())
    } else {
        (0u64, ())
    };
    Some(PumpswapBuyEvent {
        base_amount_out,
        quote_amount_in_with_lp_fee,
        protocol_fee,
        coin_creator_fee,
        pool,
        user,
        user_base_token_account,
        user_quote_token_account,
        protocol_fee_recipient,
        protocol_fee_recipient_token_account,
    })
}

fn decode_sell_event(data: &[u8]) -> Option<PumpswapSellEvent> {
    let mut r = BinaryReader::new(data);
    let _timestamp = r.read_i64_le().ok()?;
    let base_amount_in = r.read_u64_le().ok()?;
    let _min_quote = r.read_u64_le().ok()?;
    let _ub = r.read_u64_le().ok()?;
    let _uq = r.read_u64_le().ok()?;
    let _pb = r.read_u64_le().ok()?;
    let _pq = r.read_u64_le().ok()?;
    let _quote_out = r.read_u64_le().ok()?;
    let _lp_bps = r.read_u64_le().ok()?;
    let _lp_fee = r.read_u64_le().ok()?;
    let _proto_bps = r.read_u64_le().ok()?;
    let protocol_fee = r.read_u64_le().ok()?;
    let _quote_out_no_lp = r.read_u64_le().ok()?;
    let user_quote_amount_out = r.read_u64_le().ok()?;
    let pool = r.read_pubkey().ok()?;
    let user = r.read_pubkey().ok()?;
    let user_base_token_account = r.read_pubkey().ok()?;
    let user_quote_token_account = r.read_pubkey().ok()?;
    let protocol_fee_recipient = r.read_pubkey().ok()?;
    let protocol_fee_recipient_token_account = r.read_pubkey().ok()?;
    let coin_creator_fee = if r.remaining() >= 48 {
        let _creator = r.read_pubkey().ok()?;
        let _bps = r.read_u64_le().ok()?;
        r.read_u64_le().ok().unwrap_or(0)
    } else {
        0
    };
    Some(PumpswapSellEvent {
        base_amount_in,
        user_quote_amount_out,
        protocol_fee,
        coin_creator_fee,
        pool,
        user,
        user_base_token_account,
        user_quote_token_account,
        protocol_fee_recipient,
        protocol_fee_recipient_token_account,
    })
}
