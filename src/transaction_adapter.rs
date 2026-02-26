//! Adapter for unified transaction data access.

use crate::constants::{
    spl_token_instruction, tokens, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID,
};
use crate::types::{
    BalanceChange, ParseConfig, PoolEventType, TokenAmount, TokenInfo, TransactionStatus,
    InnerInstructionSet, RawInstruction, SolanaTransactionInput, TokenBalanceInput,
};
use crate::utils::{convert_to_ui_amount, get_transfer_token_mint};
use std::collections::HashMap;

pub struct TransactionAdapter<'a> {
    tx: &'a SolanaTransactionInput,
    pub config: Option<ParseConfig>,
    pub account_keys: Vec<String>,
    pub spl_token_map: HashMap<String, TokenInfo>,
    pub spl_decimals_map: HashMap<String, u8>,
}

impl<'a> TransactionAdapter<'a> {
    pub fn new(tx: &'a SolanaTransactionInput, config: Option<ParseConfig>) -> Self {
        let account_keys = Self::extract_account_keys(tx);
        let mut adapter = Self {
            tx,
            config,
            account_keys: account_keys.clone(),
            spl_token_map: HashMap::new(),
            spl_decimals_map: HashMap::new(),
        };
        adapter.extract_token_info();
        adapter
    }

    fn extract_account_keys(tx: &SolanaTransactionInput) -> Vec<String> {
        let mut keys = tx.account_keys.clone();
        if let Some(ref meta) = tx.meta {
            if let Some(ref loaded) = meta.loaded_addresses {
                keys.extend(loaded.writable.clone());
                keys.extend(loaded.readonly.clone());
            }
        }
        keys
    }

    pub fn slot(&self) -> u64 {
        self.tx.slot
    }

    pub fn block_time(&self) -> i64 {
        self.tx.block_time.unwrap_or(0)
    }

    pub fn signature(&self) -> String {
        if let Some(sig) = self.tx.signatures.first() {
            bs58::encode(sig).into_string()
        } else {
            String::new()
        }
    }

    pub fn signer(&self) -> String {
        self.get_account_key(0).unwrap_or_else(|| String::new())
    }

    pub fn signers(&self) -> Vec<String> {
        let n = 1.min(self.account_keys.len());
        self.account_keys.iter().take(n).cloned().collect()
    }

    pub fn fee(&self) -> TokenAmount {
        let fee = self.tx.meta.as_ref().and_then(|m| m.fee).unwrap_or(0);
        TokenAmount {
            amount: fee.to_string(),
            ui_amount: Some(convert_to_ui_amount(fee, 9)),
            decimals: 9,
        }
    }

    pub fn compute_units(&self) -> u64 {
        self.tx
            .meta
            .as_ref()
            .and_then(|m| m.compute_units_consumed)
            .unwrap_or(0)
    }

    pub fn tx_status(&self) -> TransactionStatus {
        match &self.tx.meta {
            None => TransactionStatus::Unknown,
            Some(m) => {
                if m.err.is_none() {
                    TransactionStatus::Success
                } else {
                    TransactionStatus::Failed
                }
            }
        }
    }

    pub fn instructions(&self) -> Vec<ParsedInstructionRef> {
        self.tx
            .instructions
            .iter()
            .enumerate()
            .map(|(i, raw)| self.raw_to_parsed(raw, i))
            .collect()
    }

    pub fn inner_instructions(&self) -> Option<&[InnerInstructionSet]> {
        self.tx.inner_instructions.as_deref()
    }

    pub fn raw_instructions(&self) -> &[RawInstruction] {
        &self.tx.instructions
    }

    pub fn raw_inner_instructions(&self) -> Option<&[InnerInstructionSet]> {
        self.tx.inner_instructions.as_deref()
    }

    fn raw_to_parsed(&self, raw: &RawInstruction, outer_index: usize) -> ParsedInstructionRef {
        let program_id = self
            .get_account_key(raw.program_id_index as usize)
            .unwrap_or_default();
        let accounts: Vec<String> = raw
            .account_key_indexes
            .iter()
            .filter_map(|&i| self.get_account_key(i as usize))
            .collect();
        ParsedInstructionRef {
            program_id,
            accounts,
            data: raw.data.clone(),
            outer_index,
        }
    }

    pub fn get_instruction(
        &self,
        raw: &RawInstruction,
        program_id_index: u8,
    ) -> crate::types::ParsedInstruction {
        let program_id = self
            .get_account_key(program_id_index as usize)
            .unwrap_or_default();
        let accounts: Vec<String> = raw
            .account_key_indexes
            .iter()
            .filter_map(|&i| self.get_account_key(i as usize))
            .collect();
        crate::types::ParsedInstruction {
            program_id,
            accounts,
            data: raw.data.clone(),
            parsed: None,
        }
    }

    pub fn get_instruction_program_id(&self, raw: &RawInstruction) -> String {
        self.get_account_key(raw.program_id_index as usize)
            .unwrap_or_default()
    }

    pub fn get_account_key(&self, index: usize) -> Option<String> {
        self.account_keys.get(index).cloned()
    }

    pub fn get_account_index(&self, address: &str) -> Option<usize> {
        self.account_keys.iter().position(|k| k == address)
    }

    pub fn is_supported_token(&self, mint: &str) -> bool {
        matches!(
            mint,
            tokens::SOL
                | tokens::USDC
                | tokens::USDT
                | tokens::USD1
                | tokens::USDG
                | tokens::PYUSD
                | tokens::EURC
                | tokens::USDY
                | tokens::FDUSD
        )
    }

    pub fn get_token_decimals(&self, mint: &str) -> u8 {
        *self.spl_decimals_map.get(mint).unwrap_or(&0)
    }

    pub fn get_pool_event_base(
        &self,
        pool_event_type: PoolEventType,
        program_id: &str,
    ) -> PoolEventBase {
        PoolEventBase {
            user: self.signer(),
            pool_event_type,
            program_id: program_id.to_string(),
            amm: crate::constants::get_program_name(program_id).to_string(),
            slot: self.slot(),
            timestamp: self.block_time(),
            signature: self.signature(),
        }
    }

    pub fn pre_balances(&self) -> Option<&[u64]> {
        self.tx.meta.as_ref()?.pre_balances.as_deref()
    }

    pub fn post_balances(&self) -> Option<&[u64]> {
        self.tx.meta.as_ref()?.post_balances.as_deref()
    }

    pub fn pre_token_balances(&self) -> Option<&[TokenBalanceInput]> {
        self.tx.meta.as_ref()?.pre_token_balances.as_deref()
    }

    pub fn post_token_balances(&self) -> Option<&[TokenBalanceInput]> {
        self.tx.meta.as_ref()?.post_token_balances.as_deref()
    }

    pub fn get_token_account_owner(&self, account_key: &str) -> Option<String> {
        let post = self.post_token_balances()?;
        let index = self.get_account_index(account_key)?;
        post.iter()
            .find(|b| b.account_index == index as u32)
            .and_then(|b| b.owner.clone())
    }

    pub fn get_token_account_balance(&self, account_keys: &[String]) -> Vec<Option<TokenAmount>> {
        account_keys
            .iter()
            .map(|key| {
                if key.is_empty() {
                    return None;
                }
                let post = self.post_token_balances()?;
                let index = self.get_account_index(key)?;
                let bal = post.iter().find(|b| b.account_index == index as u32)?;
                Some(TokenAmount {
                    amount: bal.ui_token_amount.amount.clone(),
                    ui_amount: bal.ui_token_amount.ui_amount,
                    decimals: bal.ui_token_amount.decimals,
                })
            })
            .collect()
    }

    pub fn get_token_account_pre_balance(
        &self,
        account_keys: &[String],
    ) -> Vec<Option<TokenAmount>> {
        account_keys
            .iter()
            .map(|key| {
                if key.is_empty() {
                    return None;
                }
                let pre = self.pre_token_balances()?;
                let index = self.get_account_index(key)?;
                let bal = pre.iter().find(|b| b.account_index == index as u32)?;
                Some(TokenAmount {
                    amount: bal.ui_token_amount.amount.clone(),
                    ui_amount: bal.ui_token_amount.ui_amount,
                    decimals: bal.ui_token_amount.decimals,
                })
            })
            .collect()
    }

    pub fn get_account_balance(&self, account_keys: &[String]) -> Vec<Option<TokenAmount>> {
        account_keys
            .iter()
            .map(|key| {
                if key.is_empty() {
                    return None;
                }
                let index = self.get_account_index(key)?;
                let post = self.post_balances()?;
                let amount = *post.get(index)?;
                Some(TokenAmount {
                    amount: amount.to_string(),
                    ui_amount: Some(convert_to_ui_amount(amount, 9)),
                    decimals: 9,
                })
            })
            .collect()
    }

    pub fn get_account_pre_balance(&self, account_keys: &[String]) -> Vec<Option<TokenAmount>> {
        account_keys
            .iter()
            .map(|key| {
                if key.is_empty() {
                    return None;
                }
                let index = self.get_account_index(key)?;
                let pre = self.pre_balances()?;
                let amount = *pre.get(index)?;
                Some(TokenAmount {
                    amount: amount.to_string(),
                    ui_amount: Some(convert_to_ui_amount(amount, 9)),
                    decimals: 9,
                })
            })
            .collect()
    }

    fn extract_token_info(&mut self) {
        self.extract_token_balances();
        self.extract_token_from_instructions();
        if !self.spl_token_map.contains_key(tokens::SOL) {
            self.spl_token_map.insert(
                tokens::SOL.to_string(),
                TokenInfo {
                    mint: tokens::SOL.to_string(),
                    amount: 0.0,
                    amount_raw: "0".to_string(),
                    decimals: 9,
                    authority: None,
                    destination: None,
                    destination_owner: None,
                    source: None,
                },
            );
        }
        self.spl_decimals_map.insert(tokens::SOL.to_string(), 9);
    }

    fn extract_token_balances(&mut self) {
        let post: Vec<_> = match self.post_token_balances() {
            Some(p) => p.to_vec(),
            None => return,
        };
        for balance in post {
            let mint = match &balance.mint {
                Some(m) => m.clone(),
                None => continue,
            };
            let account_key = self
                .get_account_key(balance.account_index as usize)
                .unwrap_or_default();
            if !self.spl_token_map.contains_key(&account_key) {
                self.spl_token_map.insert(
                    account_key,
                    TokenInfo {
                        mint: mint.clone(),
                        amount: balance.ui_token_amount.ui_amount.unwrap_or(0.0),
                        amount_raw: balance.ui_token_amount.amount.clone(),
                        decimals: balance.ui_token_amount.decimals,
                        authority: None,
                        destination: None,
                        destination_owner: balance.owner.clone(),
                        source: None,
                    },
                );
            }
            self.spl_decimals_map
                .insert(mint, balance.ui_token_amount.decimals);
        }
    }

    fn extract_token_from_instructions(&mut self) {
        for raw in &self.tx.instructions {
            self.extract_from_compiled_transfer(raw);
        }
        if let Some(inner) = &self.tx.inner_instructions {
            for set in inner {
                for raw in &set.instructions {
                    self.extract_from_compiled_transfer(raw);
                }
            }
        }
    }

    fn extract_from_compiled_transfer(&mut self, ix: &RawInstruction) {
        let data = &ix.data;
        if data.is_empty() {
            return;
        }
        let program_id = self
            .get_account_key(ix.program_id_index as usize)
            .unwrap_or_default();
        if program_id != TOKEN_PROGRAM_ID && program_id != TOKEN_2022_PROGRAM_ID {
            return;
        }
        let accounts: Vec<String> = ix
            .account_key_indexes
            .iter()
            .filter_map(|&i| self.get_account_key(i as usize))
            .collect();
        let (source, destination, mint, decimals) = match data[0] {
            spl_token_instruction::TRANSFER => {
                if accounts.len() < 2 {
                    return;
                }
                let source = accounts[0].clone();
                let dest = accounts[1].clone();
                let token1 = self.spl_token_map.get(&dest).map(|t| t.mint.clone());
                let token2 = self.spl_token_map.get(&source).map(|t| t.mint.clone());
                let mint =
                    get_transfer_token_mint(token1.as_deref(), token2.as_deref());
                (Some(source), Some(dest), mint, None)
            }
            spl_token_instruction::TRANSFER_CHECKED => {
                if accounts.len() < 3 {
                    return;
                }
                let dec = if data.len() >= 10 { Some(data[9]) } else { None };
                (
                    Some(accounts[0].clone()),
                    Some(accounts[2].clone()),
                    Some(accounts[1].clone()),
                    dec,
                )
            }
            spl_token_instruction::MINT_TO | spl_token_instruction::MINT_TO_CHECKED => {
                if accounts.len() < 2 {
                    return;
                }
                let dec = if data.len() >= 10 { Some(data[9]) } else { None };
                (
                    None,
                    Some(accounts[1].clone()),
                    Some(accounts[0].clone()),
                    dec,
                )
            }
            spl_token_instruction::BURN | spl_token_instruction::BURN_CHECKED => {
                if accounts.len() < 2 {
                    return;
                }
                let dec = if data.len() >= 10 { Some(data[9]) } else { None };
                (
                    Some(accounts[0].clone()),
                    None,
                    Some(accounts[1].clone()),
                    dec,
                )
            }
            _ => return,
        };
        if let Some(m) = &mint {
            if let Some(d) = decimals {
                self.spl_decimals_map.insert(m.clone(), d);
            }
        }
        for acc in [source, destination].into_iter().flatten() {
            if !self.spl_token_map.contains_key(&acc) {
                self.spl_token_map.insert(
                    acc,
                    TokenInfo {
                        mint: mint.clone().unwrap_or_else(|| tokens::SOL.to_string()),
                        amount: 0.0,
                        amount_raw: "0".to_string(),
                        decimals: decimals.unwrap_or(9),
                        authority: None,
                        destination: None,
                        destination_owner: None,
                        source: None,
                    },
                );
            }
        }
    }

    pub fn get_account_sol_balance_changes(
        &self,
        _is_owner: bool,
    ) -> HashMap<String, BalanceChange> {
        let mut changes = HashMap::new();
        let pre = self.pre_balances().unwrap_or(&[]);
        let post = self.post_balances().unwrap_or(&[]);
        for (index, key) in self.account_keys.iter().enumerate() {
            let pre_bal = pre.get(index).copied().unwrap_or(0);
            let post_bal = post.get(index).copied().unwrap_or(0);
            let change = post_bal as i64 - pre_bal as i64;
            if change != 0 {
                changes.insert(
                    key.clone(),
                    BalanceChange {
                        pre: TokenAmount {
                            amount: pre_bal.to_string(),
                            ui_amount: Some(convert_to_ui_amount(pre_bal, 9)),
                            decimals: 9,
                        },
                        post: TokenAmount {
                            amount: post_bal.to_string(),
                            ui_amount: Some(convert_to_ui_amount(post_bal, 9)),
                            decimals: 9,
                        },
                        change: TokenAmount {
                            amount: change.abs().to_string(),
                            ui_amount: Some(convert_to_ui_amount(change.unsigned_abs(), 9)),
                            decimals: 9,
                        },
                    },
                );
            }
        }
        changes
    }

    pub fn get_account_token_balance_changes(
        &self,
        _is_owner: bool,
    ) -> HashMap<String, HashMap<String, BalanceChange>> {
        let mut changes: HashMap<String, HashMap<String, BalanceChange>> = HashMap::new();
        let pre = self.pre_token_balances().unwrap_or(&[]);
        let post = self.post_token_balances().unwrap_or(&[]);
        for balance in pre {
            let key = self
                .get_account_key(balance.account_index as usize)
                .unwrap_or_default();
            let mint = balance.mint.clone().unwrap_or_default();
            if mint.is_empty() {
                continue;
            }
            changes.entry(key).or_default().insert(
                mint.clone(),
                BalanceChange {
                    pre: TokenAmount {
                        amount: balance.ui_token_amount.amount.clone(),
                        ui_amount: balance.ui_token_amount.ui_amount,
                        decimals: balance.ui_token_amount.decimals,
                    },
                    post: TokenAmount {
                        amount: "0".to_string(),
                        ui_amount: Some(0.0),
                        decimals: balance.ui_token_amount.decimals,
                    },
                    change: TokenAmount {
                        amount: "0".to_string(),
                        ui_amount: Some(0.0),
                        decimals: balance.ui_token_amount.decimals,
                    },
                },
            );
        }
        for balance in post {
            let key = self
                .get_account_key(balance.account_index as usize)
                .unwrap_or_default();
            let mint = balance.mint.clone().unwrap_or_default();
            if mint.is_empty() {
                continue;
            }
            let entry = changes.entry(key).or_default();
            if let Some(existing) = entry.get_mut(&mint) {
                existing.post = TokenAmount {
                    amount: balance.ui_token_amount.amount.clone(),
                    ui_amount: balance.ui_token_amount.ui_amount,
                    decimals: balance.ui_token_amount.decimals,
                };
                let pre_amount: u128 = existing.pre.amount.parse().unwrap_or(0);
                let post_amount: u128 = balance.ui_token_amount.amount.parse().unwrap_or(0);
                let change_amount = post_amount as i128 - pre_amount as i128;
                existing.change = TokenAmount {
                    amount: change_amount.abs().to_string(),
                    ui_amount: Some(
                        (balance.ui_token_amount.ui_amount.unwrap_or(0.0))
                            - (existing.pre.ui_amount.unwrap_or(0.0)),
                    ),
                    decimals: balance.ui_token_amount.decimals,
                };
                if change_amount == 0 {
                    entry.remove(&mint);
                }
            } else {
                entry.insert(
                    mint,
                    BalanceChange {
                        pre: TokenAmount {
                            amount: "0".to_string(),
                            ui_amount: Some(0.0),
                            decimals: balance.ui_token_amount.decimals,
                        },
                        post: TokenAmount {
                            amount: balance.ui_token_amount.amount.clone(),
                            ui_amount: balance.ui_token_amount.ui_amount,
                            decimals: balance.ui_token_amount.decimals,
                        },
                        change: TokenAmount {
                            amount: balance.ui_token_amount.amount.clone(),
                            ui_amount: balance.ui_token_amount.ui_amount,
                            decimals: balance.ui_token_amount.decimals,
                        },
                    },
                );
            }
        }
        changes
    }
}

pub struct ParsedInstructionRef {
    pub program_id: String,
    pub accounts: Vec<String>,
    pub data: Vec<u8>,
    pub outer_index: usize,
}

pub struct PoolEventBase {
    pub user: String,
    pub pool_event_type: PoolEventType,
    pub program_id: String,
    pub amm: String,
    pub slot: u64,
    pub timestamp: i64,
    pub signature: String,
}
