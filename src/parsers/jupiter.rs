//! Jupiter DEX parser (route swap events).

use borsh::BorshDeserialize;
use crate::constants::{dex_programs, discriminators};
use crate::transaction_adapter::TransactionAdapter;
use crate::transaction_utils::TransactionUtils;
use crate::types::{ClassifiedInstruction, DexInfo, TradeInfo, TransferData};
use crate::constants::get_program_name;
use crate::utils::{convert_to_ui_amount, get_trade_type};
use std::collections::HashMap;

#[derive(BorshDeserialize)]
pub struct JupiterSwapLayout {
    pub amm: [u8; 32],
    pub input_mint: [u8; 32],
    pub input_amount: u64,
    pub output_mint: [u8; 32],
    pub output_amount: u64,
}

pub struct JupiterParser<'a> {
    adapter: &'a TransactionAdapter<'a>,
    dex_info: DexInfo,
    _transfer_actions: HashMap<String, Vec<TransferData>>,
    classified_instructions: Vec<ClassifiedInstruction>,
}

impl<'a> JupiterParser<'a> {
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
        let utils = TransactionUtils::new(self.adapter);
        for ci in &self.classified_instructions {
            if ci.program_id != dex_programs::JUPITER.id {
                continue;
            }
            if self.is_jupiter_route_event_instruction(&ci.instruction.data) {
                let idx = format!("{}-{}", ci.outer_index, ci.inner_index.unwrap_or(0));
                if let Some(event) = self.parse_jupiter_route_event(&ci.instruction.data, &idx) {
                    let trade = self.build_trade_from_event(event, &utils);
                    if let Some(t) = trade {
                        trades.push(t);
                    }
                }
            }
        }
        trades
    }

    fn is_jupiter_route_event_instruction(&self, data: &[u8]) -> bool {
        if data.len() < 16 {
            return false;
        }
        data[0..16] == discriminators::JUPITER_ROUTE_EVENT
    }

    fn parse_jupiter_route_event(
        &self,
        data: &[u8],
        idx: &str,
    ) -> Option<JupiterSwapEventData> {
        if data.len() < 16 {
            return None;
        }
        let event_data = &data[16..];
        let layout: JupiterSwapLayout = BorshDeserialize::try_from_slice(event_data).ok()?;
        let input_mint = bs58::encode(layout.input_mint).into_string();
        let output_mint = bs58::encode(layout.output_mint).into_string();
        let amm = bs58::encode(layout.amm).into_string();
        Some(JupiterSwapEventData {
            amm,
            input_mint: input_mint.clone(),
            input_amount: layout.input_amount,
            output_mint: output_mint.clone(),
            output_amount: layout.output_amount,
            input_mint_decimals: self.adapter.get_token_decimals(&input_mint),
            output_mint_decimals: self.adapter.get_token_decimals(&output_mint),
            idx: idx.to_string(),
        })
    }

    fn build_trade_from_event(
        &self,
        event: JupiterSwapEventData,
        _utils: &TransactionUtils<'a>,
    ) -> Option<TradeInfo> {
        let signer_index = if self.adapter.account_keys.contains(&dex_programs::JUPITER_DCA.id.to_string()) {
            2
        } else {
            0
        };
        let user = self.adapter.get_account_key(signer_index).unwrap_or_else(|| self.adapter.signer());
        Some(TradeInfo {
            user: user.clone(),
            trade_type: get_trade_type(&event.input_mint, &event.output_mint),
            pool: vec![],
            input_token: crate::types::TokenInfo {
                mint: event.input_mint.clone(),
                amount: convert_to_ui_amount(event.input_amount, event.input_mint_decimals),
                amount_raw: event.input_amount.to_string(),
                decimals: event.input_mint_decimals,
                authority: None,
                destination: None,
                destination_owner: None,
                source: None,
            },
            output_token: crate::types::TokenInfo {
                mint: event.output_mint.clone(),
                amount: convert_to_ui_amount(event.output_amount, event.output_mint_decimals),
                amount_raw: event.output_amount.to_string(),
                decimals: event.output_mint_decimals,
                authority: None,
                destination: None,
                destination_owner: None,
                source: None,
            },
            slippage_bps: None,
            fee: None,
            fees: None,
            program_id: self.dex_info.program_id.clone(),
            amm: Some(get_program_name(&event.amm).to_string()),
            amms: Some(vec![get_program_name(&event.amm).to_string()]),
            route: self.dex_info.route.clone().or_else(|| Some("Jupiter".to_string())),
            slot: self.adapter.slot(),
            timestamp: self.block_time(),
            signature: self.adapter.signature(),
            idx: event.idx,
            signer: Some(self.adapter.signers()),
        })
    }
}

impl<'a> JupiterParser<'a> {
    fn block_time(&self) -> i64 {
        self.adapter.block_time()
    }
}

struct JupiterSwapEventData {
    amm: String,
    input_mint: String,
    input_amount: u64,
    output_mint: String,
    output_amount: u64,
    input_mint_decimals: u8,
    output_mint_decimals: u8,
    idx: String,
}
