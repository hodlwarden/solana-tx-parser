//! Solana DEX transaction parser — parse trades, liquidity, and meme events from Solana transactions.
//!
//! Build a [SolanaTransactionInput] from RPC/Geyser data (account keys, instructions, meta),
//! then use [DexParser] to parse trades (Jupiter, Raydium, Meteora, Orca, Pumpfun, etc.).

pub mod binary_reader;
pub mod constants;
pub mod dex_parser;
pub mod instruction_classifier;
pub mod parsers;
pub mod shred_parser;
pub mod transaction_adapter;
pub mod transaction_utils;
pub mod types;
pub mod utils;

pub use dex_parser::DexParser;
pub use shred_parser::ShredParser;
pub use types::{
    BalanceChange, ClassifiedInstruction, DexInfo, MemeEvent, ParseConfig, ParseResult,
    ParseShredResult, PoolEvent, SolanaTransactionInput, TokenAmount, TokenInfo, TradeInfo,
    TradeType, TransferData, TransactionStatus,
};
pub use types::{
    InnerInstructionSet, RawInstruction, TokenBalanceInput, TransactionMetaInput,
    UiTokenAmountInput,
};
