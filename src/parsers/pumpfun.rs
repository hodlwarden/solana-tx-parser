//! Pumpfun DEX parser - event-based trade parsing.

use crate::binary_reader::BinaryReader;
use crate::constants::{dex_programs, discriminators, tokens};
use crate::transaction_adapter::TransactionAdapter;
use crate::types::{ClassifiedInstruction, DexInfo, TradeInfo, TransferData};
use crate::utils::convert_to_ui_amount;
use std::collections::HashMap;

pub struct PumpfunParser<'a> {
    adapter: &'a TransactionAdapter<'a>,
    dex_info: DexInfo,
    _transfer_actions: HashMap<String, Vec<TransferData>>,
    classified_instructions: Vec<ClassifiedInstruction>,
}

impl<'a> PumpfunParser<'a> {
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
            if ci.program_id != dex_programs::PUMP_FUN.id {
                continue;
            }
            let data = &ci.instruction.data;
            if data.len() < 16 {
                continue;
            }
            if data[0..16] != discriminators::PUMPFUN_TRADE_EVENT {
                continue;
            }
            let event_data = &data[16..];
            if let Some(evt) = decode_trade_event(event_data) {
                let idx = format!("{}-{}", ci.outer_index, ci.inner_index.unwrap_or(0));
                let (input_mint, input_amount, input_dec, output_mint, output_amount, output_dec) = if evt.is_buy {
                    (
                        tokens::SOL.to_string(),
                        evt.sol_amount,
                        9u8,
                        evt.mint.clone(),
                        evt.token_amount,
                        6u8,
                    )
                } else {
                    (
                        evt.mint.clone(),
                        evt.token_amount,
                        6u8,
                        tokens::SOL.to_string(),
                        evt.sol_amount,
                        9u8,
                    )
                };
                let trade_type = if evt.is_buy {
                    crate::types::TradeType::Buy
                } else {
                    crate::types::TradeType::Sell
                };
                let mut trade = TradeInfo {
                    user: evt.user.clone(),
                    trade_type,
                    pool: evt.bonding_curve.clone().map(|p| vec![p]).unwrap_or_default(),
                    input_token: crate::types::TokenInfo {
                        mint: input_mint.clone(),
                        amount: convert_to_ui_amount(input_amount, input_dec),
                        amount_raw: input_amount.to_string(),
                        decimals: input_dec,
                        authority: None,
                        destination: None,
                        destination_owner: None,
                        source: None,
                    },
                    output_token: crate::types::TokenInfo {
                        mint: output_mint.clone(),
                        amount: convert_to_ui_amount(output_amount, output_dec),
                        amount_raw: output_amount.to_string(),
                        decimals: output_dec,
                        authority: None,
                        destination: None,
                        destination_owner: None,
                        source: None,
                    },
                    slippage_bps: None,
                    fee: None,
                    fees: None,
                    program_id: Some(dex_programs::PUMP_FUN.id.to_string()),
                    amm: Some(dex_programs::PUMP_FUN.name.to_string()),
                    amms: None,
                    route: self.dex_info.route.clone(),
                    slot: self.adapter.slot(),
                    timestamp: evt.timestamp,
                    signature: self.adapter.signature(),
                    idx: idx.clone(),
                    signer: Some(self.adapter.signers()),
                };
                if let Some(fee) = evt.fee {
                    trade.fee = Some(crate::types::FeeInfo {
                        mint: tokens::SOL.to_string(),
                        amount: convert_to_ui_amount(fee, 9),
                        amount_raw: fee.to_string(),
                        decimals: 9,
                        dex: None,
                        type_: None,
                        recipient: None,
                    });
                }
                trades.push(trade);
            }
        }
        trades
    }
}

struct PumpfunTradeEvent {
    mint: String,
    sol_amount: u64,
    token_amount: u64,
    is_buy: bool,
    user: String,
    timestamp: i64,
    bonding_curve: Option<String>,
    fee: Option<u64>,
}

fn decode_trade_event(data: &[u8]) -> Option<PumpfunTradeEvent> {
    let mut reader = BinaryReader::new(data);
    let mint_slice = reader.read_fixed_array(32).ok()?;
    let mint = bs58::encode(mint_slice).into_string();
    let sol_amount = reader.read_u64_le().ok()?;
    let token_amount = reader.read_u64_le().ok()?;
    let is_buy = reader.read_u8().ok()? == 1;
    let user_slice = reader.read_fixed_array(32).ok()?;
    let user = bs58::encode(user_slice).into_string();
    let timestamp = reader.read_i64_le().ok()?;
    let _ = reader.read_u64_le().ok()?;
    let _ = reader.read_u64_le().ok()?;
    let mut fee = None;
    if reader.remaining() >= 58 {
        let _ = reader.read_u64_le().ok()?;
        let _ = reader.read_u64_le().ok()?;
        let _ = reader.read_fixed_array(32).ok()?;
        let _ = reader.read_u16_le().ok()?;
        fee = Some(reader.read_u64_le().ok()?);
    }
    Some(PumpfunTradeEvent {
        mint,
        sol_amount,
        token_amount,
        is_buy,
        user,
        timestamp,
        bonding_curve: None,
        fee,
    })
}
