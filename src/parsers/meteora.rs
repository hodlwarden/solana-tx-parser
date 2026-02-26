//! Meteora DEX parser (DLMM, DAMM, DAMM V2) — transfer-based swap detection.

use crate::constants::{dex_programs, get_program_name, discriminators};
use crate::transaction_adapter::TransactionAdapter;
use crate::transaction_utils::TransactionUtils;
use crate::types::{ClassifiedInstruction, DexInfo, TradeInfo, TransferData};
use std::collections::HashMap;

pub struct MeteoraParser<'a> {
    adapter: &'a TransactionAdapter<'a>,
    dex_info: DexInfo,
    transfer_actions: HashMap<String, Vec<TransferData>>,
    classified_instructions: Vec<ClassifiedInstruction>,
}

impl<'a> MeteoraParser<'a> {
    pub fn new(
        adapter: &'a TransactionAdapter<'a>,
        dex_info: DexInfo,
        transfer_actions: HashMap<String, Vec<TransferData>>,
        classified_instructions: Vec<ClassifiedInstruction>,
    ) -> Self {
        Self {
            adapter,
            dex_info,
            transfer_actions,
            classified_instructions,
        }
    }

    pub fn process_trades(&self) -> Vec<TradeInfo> {
        let mut trades = Vec::new();
        let utils = TransactionUtils::new(self.adapter);
        let meteora_ids = [
            dex_programs::METEORA.id,
            dex_programs::METEORA_DAMM.id,
            dex_programs::METEORA_DAMM_V2.id,
        ];
        for ci in &self.classified_instructions {
            if !meteora_ids.contains(&ci.program_id.as_str()) {
                continue;
            }
            if !self.not_liquidity_event(&ci.instruction.data) {
                continue;
            }
            let mut transfers = TransactionUtils::get_transfers_for_instruction(
                &self.transfer_actions,
                &ci.program_id,
                ci.outer_index,
                ci.inner_index,
            );
            if transfers.len() >= 2 {
                if ci.program_id == dex_programs::METEORA.id {
                    transfers = transfers.into_iter().take(2).collect();
                }
                let dex_info = DexInfo {
                    amm: Some(get_program_name(&ci.program_id).to_string()),
                    ..self.dex_info.clone()
                };
                if let Some(mut trade) = utils.process_swap_data(&transfers, &dex_info, true) {
                    if let Some(pool) = self.get_pool_address(&ci.instruction.accounts, &ci.program_id) {
                        trade.pool = vec![pool];
                    }
                    trades.push(trade);
                }
            }
        }
        trades
    }

    fn not_liquidity_event(&self, data: &[u8]) -> bool {
        if data.len() < 8 {
            return true;
        }
        let slice8 = &data[0..8];
        if slice8 == discriminators::METEORA_DLMM_ADD_LIQUIDITY
            || slice8 == discriminators::METEORA_DLMM_REMOVE_LIQUIDITY
            || slice8 == discriminators::METEORA_DAMM_CREATE
            || slice8 == discriminators::METEORA_DAMM_ADD
            || slice8 == discriminators::METEORA_DAMM_REMOVE
            || slice8 == discriminators::METEORA_DAMM_V2_INIT
            || slice8 == discriminators::METEORA_DAMM_V2_ADD
            || slice8 == discriminators::METEORA_DAMM_V2_REMOVE
        {
            return false;
        }
        true
    }

    fn get_pool_address(&self, accounts: &[String], program_id: &str) -> Option<String> {
        if accounts.len() <= 5 {
            return None;
        }
        match program_id {
            id if id == dex_programs::METEORA.id || id == dex_programs::METEORA_DAMM.id => {
                accounts.get(0).cloned()
            }
            id if id == dex_programs::METEORA_DAMM_V2.id => accounts.get(1).cloned(),
            _ => None,
        }
    }
}
