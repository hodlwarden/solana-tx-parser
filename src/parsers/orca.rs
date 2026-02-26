//! Orca DEX parser — transfer-based swap detection.

use crate::constants::{dex_programs, get_program_name, discriminators};
use crate::transaction_adapter::TransactionAdapter;
use crate::transaction_utils::TransactionUtils;
use crate::types::{ClassifiedInstruction, DexInfo, TradeInfo, TransferData};
use std::collections::HashMap;

pub struct OrcaParser<'a> {
    adapter: &'a TransactionAdapter<'a>,
    dex_info: DexInfo,
    transfer_actions: HashMap<String, Vec<TransferData>>,
    classified_instructions: Vec<ClassifiedInstruction>,
}

impl<'a> OrcaParser<'a> {
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
        for ci in &self.classified_instructions {
            if ci.program_id != dex_programs::ORCA.id {
                continue;
            }
            if !self.not_liquidity_event(&ci.instruction.data) {
                continue;
            }
            let transfers = TransactionUtils::get_transfers_for_instruction(
                &self.transfer_actions,
                &ci.program_id,
                ci.outer_index,
                ci.inner_index,
            );
            if transfers.len() >= 2 {
                let dex_info = DexInfo {
                    amm: Some(get_program_name(&ci.program_id).to_string()),
                    ..self.dex_info.clone()
                };
                if let Some(trade) = utils.process_swap_data(&transfers, &dex_info, true) {
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
        let slice = &data[0..8];
        if slice == discriminators::ORCA_CREATE
            || slice == discriminators::ORCA_CREATE2
            || slice == discriminators::ORCA_INCREASE_LIQUIDITY
            || slice == discriminators::ORCA_INCREASE_LIQUIDITY2
            || slice == discriminators::ORCA_DECREASE_LIQUIDITY
        {
            return false;
        }
        true
    }
}
