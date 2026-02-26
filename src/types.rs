//! Types for trades, pools, transfers, and parse results.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAmount {
    pub amount: String,
    pub ui_amount: Option<f64>,
    pub decimals: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub mint: String,
    pub amount: f64,
    pub amount_raw: String,
    pub decimals: u8,
    pub authority: Option<String>,
    pub destination: Option<String>,
    pub destination_owner: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceChange {
    pub pre: TokenAmount,
    pub post: TokenAmount,
    pub change: TokenAmount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionStatus {
    Unknown,
    Success,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TradeType {
    Buy,
    Sell,
    Swap,
    Create,
    Migrate,
    Complete,
    Add,
    Remove,
    Lock,
    Burn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PoolEventType {
    Create,
    Add,
    Remove,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferInfoInner {
    pub authority: Option<String>,
    pub destination: String,
    pub destination_owner: Option<String>,
    pub mint: String,
    pub source: String,
    pub token_amount: TokenAmount,
    pub source_balance: Option<TokenAmount>,
    pub source_pre_balance: Option<TokenAmount>,
    pub destination_balance: Option<TokenAmount>,
    pub destination_pre_balance: Option<TokenAmount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferData {
    pub transfer_type: String, // "transfer" | "transferChecked" | etc
    pub program_id: String,
    pub info: TransferInfoInner,
    pub idx: String,
    pub timestamp: i64,
    pub signature: String,
    pub is_fee: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeInfo {
    pub mint: String,
    pub amount: f64,
    pub amount_raw: String,
    pub decimals: u8,
    pub dex: Option<String>,
    pub type_: Option<String>,
    pub recipient: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeInfo {
    pub user: String,
    pub trade_type: TradeType,
    pub pool: Vec<String>,
    pub input_token: TokenInfo,
    pub output_token: TokenInfo,
    pub slippage_bps: Option<u64>,
    pub fee: Option<FeeInfo>,
    pub fees: Option<Vec<FeeInfo>>,
    pub program_id: Option<String>,
    pub amm: Option<String>,
    pub amms: Option<Vec<String>>,
    pub route: Option<String>,
    pub slot: u64,
    pub timestamp: i64,
    pub signature: String,
    pub idx: String,
    pub signer: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolEvent {
    pub user: String,
    pub pool_event_type: PoolEventType,
    pub program_id: Option<String>,
    pub amm: Option<String>,
    pub slot: u64,
    pub timestamp: i64,
    pub signature: String,
    pub idx: String,
    pub signer: Option<Vec<String>>,
    pub pool_id: String,
    pub config: Option<String>,
    pub pool_lp_mint: Option<String>,
    pub token0_mint: Option<String>,
    pub token0_amount: Option<f64>,
    pub token0_amount_raw: Option<String>,
    pub token0_balance_change: Option<String>,
    pub token0_decimals: Option<u8>,
    pub token1_mint: Option<String>,
    pub token1_amount: Option<f64>,
    pub token1_amount_raw: Option<String>,
    pub token1_balance_change: Option<String>,
    pub token1_decimals: Option<u8>,
    pub lp_amount: Option<f64>,
    pub lp_amount_raw: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemeEvent {
    pub event_type: TradeType,
    pub timestamp: i64,
    pub idx: String,
    pub slot: u64,
    pub signature: String,
    pub user: String,
    pub base_mint: String,
    pub quote_mint: String,
    pub input_token: Option<TokenInfo>,
    pub output_token: Option<TokenInfo>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub uri: Option<String>,
    pub decimals: Option<u8>,
    pub total_supply: Option<f64>,
    pub fee: Option<f64>,
    pub protocol_fee: Option<f64>,
    pub platform_fee: Option<f64>,
    pub creator: Option<String>,
    pub bonding_curve: Option<String>,
    pub pool: Option<String>,
    pub protocol: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ClassifiedInstruction {
    pub instruction: ParsedInstruction,
    pub program_id: String,
    pub outer_index: usize,
    pub inner_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct ParsedInstruction {
    pub program_id: String,
    pub accounts: Vec<String>,
    pub data: Vec<u8>,
    pub parsed: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default)]
pub struct DexInfo {
    pub program_id: Option<String>,
    pub amm: Option<String>,
    pub route: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    pub state: bool,
    pub fee: TokenAmount,
    pub aggregate_trade: Option<TradeInfo>,
    pub trades: Vec<TradeInfo>,
    pub liquidities: Vec<PoolEvent>,
    pub transfers: Vec<TransferData>,
    pub meme_events: Vec<MemeEvent>,
    pub slot: u64,
    pub timestamp: i64,
    pub signature: String,
    pub signer: Vec<String>,
    pub compute_units: u64,
    pub tx_status: TransactionStatus,
    pub msg: Option<String>,
    pub sol_balance_change: Option<BalanceChange>,
    pub token_balance_change: Option<std::collections::HashMap<String, BalanceChange>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseShredResult {
    pub state: bool,
    pub signature: String,
    pub instructions: std::collections::HashMap<String, Vec<serde_json::Value>>,
    pub msg: Option<String>,
}

/// Configuration for parsing
#[derive(Debug, Clone, Default)]
pub struct ParseConfig {
    pub try_unknown_dex: bool,
    pub program_ids: Option<Vec<String>>,
    pub ignore_program_ids: Option<Vec<String>>,
    pub throw_error: bool,
    pub aggregate_trades: bool,
}

/// Input transaction + meta. Build from RPC getTransaction or Geyser.
#[derive(Debug, Clone)]
pub struct SolanaTransactionInput {
    pub slot: u64,
    pub block_time: Option<i64>,
    pub version: Option<u8>,
    pub signatures: Vec<Vec<u8>>,
    pub account_keys: Vec<String>,
    pub instructions: Vec<RawInstruction>,
    pub inner_instructions: Option<Vec<InnerInstructionSet>>,
    pub meta: Option<TransactionMetaInput>,
}

#[derive(Debug, Clone)]
pub struct RawInstruction {
    pub program_id_index: u8,
    pub data: Vec<u8>,
    pub account_key_indexes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct InnerInstructionSet {
    pub index: u32,
    pub instructions: Vec<RawInstruction>,
}

#[derive(Debug, Clone)]
pub struct TransactionMetaInput {
    pub err: Option<serde_json::Value>,
    pub fee: Option<u64>,
    pub pre_balances: Option<Vec<u64>>,
    pub post_balances: Option<Vec<u64>>,
    pub pre_token_balances: Option<Vec<TokenBalanceInput>>,
    pub post_token_balances: Option<Vec<TokenBalanceInput>>,
    pub inner_instructions: Option<Vec<InnerInstructionSet>>,
    pub loaded_addresses: Option<LoadedAddressesInput>,
    pub compute_units_consumed: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct TokenBalanceInput {
    pub account_index: u32,
    pub mint: Option<String>,
    pub owner: Option<String>,
    pub ui_token_amount: UiTokenAmountInput,
}

#[derive(Debug, Clone)]
pub struct UiTokenAmountInput {
    pub amount: String,
    pub decimals: u8,
    pub ui_amount: Option<f64>,
    pub ui_amount_string: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LoadedAddressesInput {
    pub writable: Vec<String>,
    pub readonly: Vec<String>,
}
