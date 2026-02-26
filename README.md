# solana-tx-parser (Rust)

Parse Solana DEX transactions for trades, liquidity events, and meme events.

## Supported DEXes

- **Jupiter** – route swap events (16-byte discriminator + Borsh layout)
- **Raydium** – V4, AMM, CPMM, CL, Route (transfer-based; excludes liquidity instructions)
- **Orca** – Whirlpool (transfer-based; excludes liquidity instructions)
- **Meteora** – DLMM, DAMM, DAMM V2 (transfer-based; excludes liquidity instructions)
- **Pumpfun** – event-based (trade event 16-byte discriminator + BinaryReader layout)
- **Pumpswap** – event-based (buy/sell event discriminators + BinaryReader layout)

## Usage

Build a `SolanaTransactionInput` from your RPC or Geyser data (account keys, compiled instructions, inner instructions, and meta), then call the parser:

```rust
use solana_dex_parser::{DexParser, SolanaTransactionInput, ParseConfig};

let tx = SolanaTransactionInput {
    slot: 123,
    block_time: Some(1_700_000_000),
    version: Some(0),
    signatures: vec![/* base58 decode of signature */],
    account_keys: vec!["...".to_string()],
    instructions: vec![/* RawInstruction { program_id_index, data, account_key_indexes } */],
    inner_instructions: Some(vec![]),
    meta: Some(TransactionMetaInput { ... }),
};

let parser = DexParser::new();
let config = ParseConfig {
    try_unknown_dex: true,
    aggregate_trades: true,
    ..Default::default()
};
let trades = parser.parse_trades(&tx, Some(config));
```

## Input format

- **account_keys**: Full list of account pubkeys (base58 strings), including from address table lookups if using versioned transactions.
- **instructions**: Each instruction is `RawInstruction { program_id_index: u8, data: Vec<u8>, account_key_indexes: Vec<u8> }` (indexes into `account_keys`).
- **inner_instructions**: Same format, with an `index` pointing to the outer instruction.
- **meta**: Optional fee, pre/post balances, pre/post token balances, loaded addresses, compute units.

You can map from Solana RPC `getTransaction` JSON or from Geyser/relayer payloads into this structure.

## License

MIT
