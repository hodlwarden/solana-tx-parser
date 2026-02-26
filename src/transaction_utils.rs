//! Transaction utilities: DEX info, transfer actions, swap data.

use crate::constants::{dex_programs, FEE_ACCOUNTS, SYSTEM_PROGRAMS};
use crate::instruction_classifier::InstructionClassifier;
use crate::transaction_adapter::TransactionAdapter;
use crate::types::{DexInfo, TransferData, TransferInfoInner};
use crate::utils::convert_to_ui_amount;
use std::collections::HashMap;

pub struct TransactionUtils<'a> {
    adapter: &'a TransactionAdapter<'a>,
}

impl<'a> TransactionUtils<'a> {
    pub fn new(adapter: &'a TransactionAdapter<'a>) -> Self {
        Self { adapter }
    }

    pub fn get_dex_info(&self, classifier: &InstructionClassifier<'a>) -> DexInfo {
        let program_ids = classifier.get_all_program_ids();
        if program_ids.is_empty() {
            return DexInfo::default();
        }
        for program_id in &program_ids {
            let id = program_id.as_str();
            if id == dex_programs::JUPITER.id {
                return DexInfo {
                    program_id: Some(program_id.clone()),
                    route: Some(dex_programs::JUPITER.name.to_string()),
                    amm: None,
                };
            }
            if id == dex_programs::JUPITER_DCA.id {
                return DexInfo {
                    program_id: Some(program_id.clone()),
                    route: Some(dex_programs::JUPITER_DCA.name.to_string()),
                    amm: None,
                };
            }
            if id == dex_programs::RAYDIUM_V4.id {
                return DexInfo {
                    program_id: Some(program_id.clone()),
                    route: None,
                    amm: Some(dex_programs::RAYDIUM_V4.name.to_string()),
                };
            }
            if id == dex_programs::METEORA.id {
                return DexInfo {
                    program_id: Some(program_id.clone()),
                    route: None,
                    amm: Some(dex_programs::METEORA.name.to_string()),
                };
            }
            if id == dex_programs::ORCA.id {
                return DexInfo {
                    program_id: Some(program_id.clone()),
                    route: None,
                    amm: Some(dex_programs::ORCA.name.to_string()),
                };
            }
            if id == dex_programs::PUMP_FUN.id {
                return DexInfo {
                    program_id: Some(program_id.clone()),
                    route: None,
                    amm: Some(dex_programs::PUMP_FUN.name.to_string()),
                };
            }
            if id == dex_programs::PUMP_SWAP.id {
                return DexInfo {
                    program_id: Some(program_id.clone()),
                    route: None,
                    amm: Some(dex_programs::PUMP_SWAP.name.to_string()),
                };
            }
        }
        DexInfo {
            program_id: program_ids.first().cloned(),
            ..Default::default()
        }
    }

    /// Build transfer actions from inner and outer instructions (compiled SPL transfer/transferChecked only).
    pub fn get_transfer_actions(
        &self,
        extra_types: &[&str],
    ) -> HashMap<String, Vec<TransferData>> {
        let mut actions: HashMap<String, Vec<TransferData>> = HashMap::new();
        // Outer instructions
        for (outer_index, raw) in self.adapter.raw_instructions().iter().enumerate() {
            let program_id = self.adapter.get_instruction_program_id(raw);
            if SYSTEM_PROGRAMS.contains(&program_id.as_str()) {
                continue;
            }
            let group_key = format!("{}:{}", program_id, outer_index);
            if let Some(transfer) = self.parse_compiled_action(raw, &outer_index.to_string(), extra_types) {
                let is_fee = FEE_ACCOUNTS.contains(&transfer.info.destination.as_str())
                    || transfer.info.destination_owner.as_ref().map(|o| FEE_ACCOUNTS.contains(&o.as_str())).unwrap_or(false);
                let mut t = transfer;
                if is_fee {
                    t.is_fee = Some(true);
                }
                actions.entry(group_key).or_default().push(t);
            }
        }
        let inner = match self.adapter.raw_inner_instructions() {
            Some(i) => i,
            None => return actions,
        };
        for set in inner {
            let outer_index = set.index as usize;
            let outer_program_id = self
                .adapter
                .raw_instructions()
                .get(outer_index)
                .map(|r| self.adapter.get_instruction_program_id(r))
                .unwrap_or_default();
            for (inner_index, raw) in set.instructions.iter().enumerate() {
                let group_key = format!("{}:{}-{}", outer_program_id, outer_index, inner_index);
                if let Some(transfer) = self.parse_compiled_action(
                    raw,
                    &format!("{}-{}", outer_index, inner_index),
                    extra_types,
                ) {
                    let is_fee = FEE_ACCOUNTS.contains(&transfer.info.destination.as_str())
                        || transfer
                            .info
                            .destination_owner
                            .as_ref()
                            .map(|o| FEE_ACCOUNTS.contains(&o.as_str()))
                            .unwrap_or(false);
                    let mut t = transfer;
                    if is_fee {
                        t.is_fee = Some(true);
                    }
                    actions.entry(group_key).or_default().push(t);
                }
            }
        }
        actions
    }

    fn parse_compiled_action(
        &self,
        raw: &crate::types::RawInstruction,
        idx: &str,
        _extra_types: &[&str],
    ) -> Option<TransferData> {
        let data = &raw.data;
        if data.is_empty() {
            return None;
        }
        let program_id = self.adapter.get_instruction_program_id(raw);
        let accounts: Vec<String> = raw
            .account_key_indexes
            .iter()
            .filter_map(|&i| self.adapter.get_account_key(i as usize))
            .collect();
        match (program_id.as_str(), data[0]) {
            (crate::constants::TOKEN_PROGRAM_ID, crate::constants::spl_token_instruction::TRANSFER) => {
                if accounts.len() < 2 {
                    return None;
                }
                let amount = if data.len() >= 9 {
                    u64::from_le_bytes(data[1..9].try_into().ok()?)
                } else {
                    return None;
                };
                let source = accounts[0].clone();
                let destination = accounts[1].clone();
                let token1 = self.adapter.spl_token_map.get(&destination).map(|t| t.mint.clone());
                let token2 = self.adapter.spl_token_map.get(&source).map(|t| t.mint.clone());
                let mint = crate::utils::get_transfer_token_mint(
                    token1.as_deref(),
                    token2.as_deref(),
                )?;
                let decimals = self.adapter.get_token_decimals(&mint);
                let (sb, db, spb, dpb) = {
                    let sb = self.adapter.get_token_account_balance(&[source.clone()]);
                    let db = self.adapter.get_token_account_balance(&[destination.clone()]);
                    let spb = self.adapter.get_token_account_pre_balance(&[source.clone()]);
                    let dpb = self.adapter.get_token_account_pre_balance(&[destination.clone()]);
                    (
                        sb.into_iter().next().flatten(),
                        db.into_iter().next().flatten(),
                        spb.into_iter().next().flatten(),
                        dpb.into_iter().next().flatten(),
                    )
                };
                Some(TransferData {
                    transfer_type: "transfer".to_string(),
                    program_id: program_id.clone(),
                    info: TransferInfoInner {
                        authority: accounts.get(2).cloned(),
                        destination,
                        destination_owner: self.adapter.get_token_account_owner(&accounts[1]),
                        mint: mint.clone(),
                        source: source.clone(),
                        token_amount: crate::types::TokenAmount {
                            amount: amount.to_string(),
                            ui_amount: Some(convert_to_ui_amount(amount, decimals)),
                            decimals,
                        },
                        source_balance: sb,
                        source_pre_balance: spb,
                        destination_balance: db,
                        destination_pre_balance: dpb,
                    },
                    idx: idx.to_string(),
                    timestamp: self.adapter.block_time(),
                    signature: self.adapter.signature(),
                    is_fee: None,
                })
            }
            (crate::constants::TOKEN_PROGRAM_ID, crate::constants::spl_token_instruction::TRANSFER_CHECKED)
            | (crate::constants::TOKEN_2022_PROGRAM_ID, crate::constants::spl_token_instruction::TRANSFER_CHECKED) => {
                if accounts.len() < 3 || data.len() < 10 {
                    return None;
                }
                let amount = u64::from_le_bytes(data[1..9].try_into().ok()?);
                let decimals = data[9];
                let source = accounts[0].clone();
                let mint = accounts[1].clone();
                let destination = accounts[2].clone();
                let (sb, db, spb, dpb) = {
                    let sb = self.adapter.get_token_account_balance(&[source.clone()]);
                    let db = self.adapter.get_token_account_balance(&[destination.clone()]);
                    let spb = self.adapter.get_token_account_pre_balance(&[source.clone()]);
                    let dpb = self.adapter.get_token_account_pre_balance(&[destination.clone()]);
                    (
                        sb.into_iter().next().flatten(),
                        db.into_iter().next().flatten(),
                        spb.into_iter().next().flatten(),
                        dpb.into_iter().next().flatten(),
                    )
                };
                Some(TransferData {
                    transfer_type: "transferChecked".to_string(),
                    program_id,
                    info: TransferInfoInner {
                        authority: accounts.get(3).cloned(),
                        destination,
                        destination_owner: self.adapter.get_token_account_owner(&accounts[2]),
                        mint,
                        source,
                        token_amount: crate::types::TokenAmount {
                            amount: amount.to_string(),
                            ui_amount: Some(convert_to_ui_amount(amount, decimals)),
                            decimals,
                        },
                        source_balance: sb,
                        source_pre_balance: spb,
                        destination_balance: db,
                        destination_pre_balance: dpb,
                    },
                    idx: idx.to_string(),
                    timestamp: self.adapter.block_time(),
                    signature: self.adapter.signature(),
                    is_fee: None,
                })
            }
            _ => None,
        }
    }

    /// Get transfers for a specific instruction (by program_id, outer_index, optional inner_index).
    pub fn get_transfers_for_instruction(
        transfer_actions: &HashMap<String, Vec<TransferData>>,
        program_id: &str,
        outer_index: usize,
        inner_index: Option<usize>,
    ) -> Vec<TransferData> {
        let key = match inner_index {
            Some(i) => format!("{}:{}-{}", program_id, outer_index, i),
            None => format!("{}:{}", program_id, outer_index),
        };
        let transfers = transfer_actions.get(&key).cloned().unwrap_or_default();
        transfers
            .into_iter()
            .filter(|t| matches!(t.transfer_type.as_str(), "transfer" | "transferChecked"))
            .collect()
    }

    /// Build TradeInfo from transfer list (swap: 2+ tokens, determine in/out by signer).
    pub fn process_swap_data(
        &self,
        transfers: &[TransferData],
        dex_info: &DexInfo,
        skip_native: bool,
    ) -> Option<crate::types::TradeInfo> {
        use crate::constants::tokens;
        if transfers.len() < 2 {
            return None;
        }
        let mut unique_mints: Vec<String> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for t in transfers {
            if skip_native && t.info.mint == tokens::NATIVE {
                continue;
            }
            if !seen.insert(t.info.mint.clone()) {
                continue;
            }
            unique_mints.push(t.info.mint.clone());
        }
        if unique_mints.len() < 2 {
            return None;
        }
        let signer = self.get_swap_signer();
        let (input_mint, output_mint, input_raw, output_raw, fee) =
            self.sum_token_amounts(transfers, &unique_mints[0], &unique_mints[unique_mints.len() - 1], &signer)?;
        let in_dec = self.adapter.get_token_decimals(&input_mint);
        let out_dec = self.adapter.get_token_decimals(&output_mint);
        let trade_type = crate::utils::get_trade_type(&input_mint, &output_mint);
        let mut trade = crate::types::TradeInfo {
            user: signer.clone(),
            trade_type,
            pool: vec![],
            input_token: crate::types::TokenInfo {
                mint: input_mint.clone(),
                amount: crate::utils::convert_to_ui_amount_u128(input_raw, in_dec),
                amount_raw: input_raw.to_string(),
                decimals: in_dec,
                authority: None,
                destination: None,
                destination_owner: None,
                source: None,
            },
            output_token: crate::types::TokenInfo {
                mint: output_mint.clone(),
                amount: crate::utils::convert_to_ui_amount_u128(output_raw, out_dec),
                amount_raw: output_raw.to_string(),
                decimals: out_dec,
                authority: None,
                destination: None,
                destination_owner: None,
                source: None,
            },
            slippage_bps: None,
            fee: None,
            fees: None,
            program_id: dex_info.program_id.clone(),
            amm: dex_info.amm.clone(),
            amms: None,
            route: dex_info.route.clone(),
            slot: self.adapter.slot(),
            timestamp: self.adapter.block_time(),
            signature: self.adapter.signature(),
            idx: transfers.first().map(|t| t.idx.clone()).unwrap_or_default(),
            signer: Some(self.adapter.signers()),
        };
        if let Some(fee_transfer) = fee {
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
        Some(trade)
    }

    fn get_swap_signer(&self) -> String {
        if self.adapter.account_keys.contains(&dex_programs::JUPITER_DCA.id.to_string()) {
            self.adapter.get_account_key(2).unwrap_or_else(|| self.adapter.signer())
        } else {
            self.adapter.signer()
        }
    }

    fn sum_token_amounts(
        &self,
        transfers: &[TransferData],
        input_mint: &str,
        output_mint: &str,
        _signer: &str,
    ) -> Option<(String, String, u128, u128, Option<TransferData>)> {
        let mut input_raw: u128 = 0;
        let mut output_raw: u128 = 0;
        let mut fee_transfer: Option<TransferData> = None;
        let mut seen = std::collections::HashSet::new();
        for t in transfers {
            let dest_owner = t.info.destination_owner.as_deref().unwrap_or("");
            if FEE_ACCOUNTS.contains(&t.info.destination.as_str()) || FEE_ACCOUNTS.contains(&dest_owner) {
                fee_transfer = Some(t.clone());
                continue;
            }
            let key = format!("{}-{}", t.info.token_amount.amount, t.info.mint);
            if !seen.insert(key) {
                continue;
            }
            let amount: u128 = t.info.token_amount.amount.parse().unwrap_or(0);
            if t.info.mint == input_mint {
                input_raw += amount;
            }
            if t.info.mint == output_mint {
                output_raw += amount;
            }
        }
        Some((
            input_mint.to_string(),
            output_mint.to_string(),
            input_raw,
            output_raw,
            fee_transfer,
        ))
    }
}
