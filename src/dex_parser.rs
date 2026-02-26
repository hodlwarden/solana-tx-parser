//! Main DEX parser: trades, liquidity, transfers, meme events.

use crate::constants::{dex_programs, get_program_name};
use crate::instruction_classifier::InstructionClassifier;
use crate::parsers::{
    jupiter::JupiterParser,
    meteora::MeteoraParser,
    orca::OrcaParser,
    pumpfun::PumpfunParser,
    pumpswap::PumpswapParser,
    raydium::RaydiumParser,
};
use crate::transaction_adapter::TransactionAdapter;
use crate::transaction_utils::TransactionUtils;
use crate::types::{ParseConfig, ParseResult, TokenAmount, TradeInfo, TransactionStatus};
use crate::utils::get_final_swap;
use std::collections::HashMap;

pub struct DexParser;

impl DexParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_trades(
        &self,
        tx: &crate::types::SolanaTransactionInput,
        config: Option<ParseConfig>,
    ) -> Vec<TradeInfo> {
        self.parse_with_classifier(tx, config.unwrap_or_default(), ParseType::Trades)
            .trades
    }

    pub fn parse_all(
        &self,
        tx: &crate::types::SolanaTransactionInput,
        config: Option<ParseConfig>,
    ) -> ParseResult {
        self.parse_with_classifier(tx, config.unwrap_or_default(), ParseType::All)
    }

    fn parse_with_classifier(
        &self,
        tx: &crate::types::SolanaTransactionInput,
        config: ParseConfig,
        parse_type: ParseType,
    ) -> ParseResult {
        let mut result = ParseResult {
            state: true,
            fee: TokenAmount {
                amount: "0".to_string(),
                ui_amount: Some(0.0),
                decimals: 9,
            },
            aggregate_trade: None,
            trades: Vec::new(),
            liquidities: Vec::new(),
            transfers: Vec::new(),
            meme_events: Vec::new(),
            slot: tx.slot,
            timestamp: tx.block_time.unwrap_or(0),
            signature: String::new(),
            signer: Vec::new(),
            compute_units: 0,
            tx_status: TransactionStatus::Unknown,
            msg: None,
            sol_balance_change: None,
            token_balance_change: None,
        };

        let adapter = TransactionAdapter::new(tx, Some(config.clone()));
        result.signature = adapter.signature();
        result.signer = adapter.signers();
        result.timestamp = adapter.block_time();
        result.compute_units = adapter.compute_units();
        result.tx_status = adapter.tx_status();
        result.fee = adapter.fee();

        let classifier = InstructionClassifier::new(&adapter);
        let utils = TransactionUtils::new(&adapter);
        let dex_info = utils.get_dex_info(&classifier);
        let all_program_ids = classifier.get_all_program_ids();

        if let Some(ref filter_ids) = config.program_ids {
            if !all_program_ids.iter().any(|id| filter_ids.contains(id)) {
                result.state = false;
                return result;
            }
        }

        let transfer_actions = utils.get_transfer_actions(&[
            "mintTo",
            "burn",
            "mintToChecked",
            "burnChecked",
        ]);

        if parse_type == ParseType::Trades || parse_type == ParseType::All {
            // Jupiter first
            let jupiter_ids = [
                dex_programs::JUPITER.id,
                dex_programs::JUPITER_DCA.id,
                dex_programs::JUPITER_VA.id,
                dex_programs::JUPITER_LIMIT_ORDER_V2.id,
            ];
            if dex_info.program_id.as_ref().map(|p| jupiter_ids.contains(&p.as_str())) == Some(true) {
                let program_id = dex_info.program_id.as_deref().unwrap_or("");
                let instructions = classifier.get_instructions(program_id);
                if program_id == dex_programs::JUPITER.id {
                    let parser = JupiterParser::new(
                        &adapter,
                        crate::types::DexInfo {
                            program_id: Some(program_id.to_string()),
                            amm: Some(get_program_name(program_id).to_string()),
                            route: dex_info.route.clone(),
                        },
                        transfer_actions.clone(),
                        instructions,
                    );
                    let trades = parser.process_trades();
                    if !trades.is_empty() {
                        if config.aggregate_trades {
                            result.aggregate_trade =
                                get_final_swap(&trades, dex_info.amm.as_deref(), dex_info.route.as_deref());
                        } else {
                            result.trades = trades;
                        }
                        if !result.trades.is_empty() || result.aggregate_trade.is_some() {
                            return result;
                        }
                    }
                }
            }

            for program_id in &all_program_ids {
                if config.program_ids.as_ref().map(|p| !p.contains(program_id)).unwrap_or(false) {
                    continue;
                }
                if config.ignore_program_ids.as_ref().map(|p| p.contains(program_id)).unwrap_or(false) {
                    continue;
                }
                let instructions = classifier.get_instructions(program_id);
                let dex_info_here = crate::types::DexInfo {
                    program_id: Some(program_id.clone()),
                    amm: Some(get_program_name(program_id).to_string()),
                    route: dex_info.route.clone(),
                };
                if program_id == dex_programs::JUPITER.id {
                    let parser = JupiterParser::new(&adapter, dex_info_here.clone(), transfer_actions.clone(), instructions);
                    result.trades.extend(parser.process_trades());
                } else if program_id == dex_programs::RAYDIUM_V4.id
                    || program_id == dex_programs::RAYDIUM_AMM.id
                    || program_id == dex_programs::RAYDIUM_CPMM.id
                    || program_id == dex_programs::RAYDIUM_CL.id
                    || program_id == dex_programs::RAYDIUM_ROUTE.id
                {
                    let parser = RaydiumParser::new(&adapter, dex_info_here.clone(), transfer_actions.clone(), instructions);
                    result.trades.extend(parser.process_trades());
                } else if program_id == dex_programs::ORCA.id {
                    let parser = OrcaParser::new(&adapter, dex_info_here.clone(), transfer_actions.clone(), instructions);
                    result.trades.extend(parser.process_trades());
                } else if program_id == dex_programs::METEORA.id
                    || program_id == dex_programs::METEORA_DAMM.id
                    || program_id == dex_programs::METEORA_DAMM_V2.id
                {
                    let parser = MeteoraParser::new(&adapter, dex_info_here.clone(), transfer_actions.clone(), instructions);
                    result.trades.extend(parser.process_trades());
                } else if program_id == dex_programs::PUMP_FUN.id {
                    let parser = PumpfunParser::new(&adapter, dex_info_here.clone(), transfer_actions.clone(), instructions);
                    result.trades.extend(parser.process_trades());
                } else if program_id == dex_programs::PUMP_SWAP.id {
                    let parser = PumpswapParser::new(&adapter, dex_info_here.clone(), transfer_actions.clone(), instructions);
                    result.trades.extend(parser.process_trades());
                }
            }

            if result.trades.len() > 1 {
                let mut seen = HashMap::new();
                result.trades.retain(|t| {
                    let key = format!("{}-{}", t.idx, t.signature);
                    if seen.insert(key.clone(), ()).is_some() {
                        false
                    } else {
                        true
                    }
                });
                if config.aggregate_trades {
                    result.aggregate_trade =
                        get_final_swap(&result.trades, dex_info.amm.as_deref(), dex_info.route.as_deref());
                }
            }
        }

        result.sol_balance_change = adapter
            .get_account_sol_balance_changes(false)
            .remove(&adapter.signer());
        let token_changes = adapter.get_account_token_balance_changes(true);
        result.token_balance_change = token_changes.get(&adapter.signer()).cloned();

        result
    }
}

impl Default for DexParser {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(PartialEq, Eq)]
enum ParseType {
    Trades,
    All,
}
