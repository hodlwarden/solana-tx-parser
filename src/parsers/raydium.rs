//! Raydium DEX parser (V4, AMM, CPMM, CL) - transfer-based swap detection.

use crate::constants::{dex_programs, discriminators, get_program_name};
use crate::transaction_adapter::TransactionAdapter;
use crate::transaction_utils::TransactionUtils;
use crate::types::{ClassifiedInstruction, DexInfo, TradeInfo, TransferData};
use std::collections::HashMap;

pub struct RaydiumParser<'a> {
    adapter: &'a TransactionAdapter<'a>,
    dex_info: DexInfo,
    transfer_actions: HashMap<String, Vec<TransferData>>,
    classified_instructions: Vec<ClassifiedInstruction>,
}

impl<'a> RaydiumParser<'a> {
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
                let take = 2.min(transfers.len());
                if let Some(mut trade) = utils.process_swap_data(&transfers[..take], &dex_info, true) {
                    if let Some(pool) = self.get_pool_address(&ci.instruction.accounts, &ci.program_id) {
                        trade.pool = vec![pool];
                    }
                    if transfers.len() > 2 {
                        let fee_transfer = &transfers[2];
                        trade.fee = Some(crate::types::FeeInfo {
                            mint: fee_transfer.info.mint.clone(),
                            amount: fee_transfer.info.token_amount.ui_amount.unwrap_or(0.0),
                            amount_raw: fee_transfer.info.token_amount.amount.clone(),
                            decimals: fee_transfer.info.token_amount.decimals,
                            dex: None,
                            type_: None,
                            recipient: None,
                        });
                    }
                    trades.push(trade);
                }
            }
        }
        trades
    }

    fn not_liquidity_event(&self, data: &[u8]) -> bool {
        if data.is_empty() {
            return true;
        }
        if data.len() >= 1 {
            if data[0] == discriminators::RAYDIUM_CREATE[0]
                || data[0] == discriminators::RAYDIUM_ADD_LIQUIDITY[0]
                || data[0] == discriminators::RAYDIUM_REMOVE_LIQUIDITY[0]
            {
                return false;
            }
        }
        if data.len() >= 8 {
            let slice = &data[0..8];
            if slice == discriminators::RAYDIUM_CPMM_CREATE
                || slice == discriminators::RAYDIUM_CPMM_ADD_LIQUIDITY
                || slice == discriminators::RAYDIUM_CPMM_REMOVE_LIQUIDITY
            {
                return false;
            }
        }
        true
    }

    fn get_pool_address(&self, accounts: &[String], program_id: &str) -> Option<String> {
        if accounts.len() <= 5 {
            return None;
        }
        match program_id {
            id if id == dex_programs::RAYDIUM_V4.id || id == dex_programs::RAYDIUM_AMM.id => {
                accounts.get(1).cloned()
            }
            id if id == dex_programs::RAYDIUM_CL.id => accounts.get(2).cloned(),
            id if id == dex_programs::RAYDIUM_CPMM.id => accounts.get(3).cloned(),
            _ => None,
        }
    }
}
