//! DEX program IDs, discriminators, tokens, and instruction types.

use std::collections::HashMap;

pub const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
pub const TOKEN_2022_PROGRAM_ID: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
pub const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

pub mod tokens {
    pub const NATIVE: &str = "11111111111111111111111111111111";
    pub const SOL: &str = "So11111111111111111111111111111111111111112";
    pub const USDC: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    pub const USDT: &str = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
    pub const USD1: &str = "USD1ttGY1N17NEEHLmELoaybftRBUSErhqYiQzvEmuB";
    pub const USDG: &str = "2u1tszSeqZ3qBWF3uNGPFc8TzMk2tdiwknnRMWGWjGWH";
    pub const PYUSD: &str = "2b1kV6DkPAnxd5ixfnxCpjxmKwqjjaYmCZfHsFu24GXo";
    pub const EURC: &str = "HzwqbKZw8HxMN6bF2yFZNrht3c2iXXzpKcFu7uBEDKtr";
    pub const USDY: &str = "A1KLoBrKBde8Ty9qtNQUtq3C2ortoC3u7twggz7sEto6";
    pub const FDUSD: &str = "9zNQRsGLjNKwCUU5Gq5LR8beUCPzQMVMqKAi3SSZh54u";
}

pub fn token_decimals(mint: &str) -> Option<u8> {
    let m: HashMap<&str, u8> = [
        (tokens::SOL, 9),
        (tokens::USDC, 6),
        (tokens::USDT, 6),
        (tokens::USD1, 6),
        (tokens::USDG, 6),
        (tokens::PYUSD, 6),
        (tokens::EURC, 6),
        (tokens::USDY, 6),
        (tokens::FDUSD, 6),
    ]
    .into_iter()
    .collect();
    m.get(mint).copied()
}

#[derive(Clone, Debug)]
pub struct DexProgram {
    pub id: &'static str,
    pub name: &'static str,
    pub tags: &'static [&'static str],
}

pub mod dex_programs {
    use super::DexProgram;

    pub const JUPITER: DexProgram = DexProgram {
        id: "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",
        name: "Jupiter",
        tags: &["route"],
    };
    pub const JUPITER_DCA: DexProgram = DexProgram {
        id: "DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M",
        name: "JupiterDCA",
        tags: &["route"],
    };
    pub const JUPITER_DCA_KEEPER1: DexProgram = DexProgram {
        id: "DCAKxn5PFNN1mBREPWGdk1RXg5aVH9rPErLfBFEi2Emb",
        name: "JupiterDcaKeeper1",
        tags: &["route"],
    };
    pub const JUPITER_DCA_KEEPER2: DexProgram = DexProgram {
        id: "DCAKuApAuZtVNYLk3KTAVW9GLWVvPbnb5CxxRRmVgcTr",
        name: "JupiterDcaKeeper2",
        tags: &["route"],
    };
    pub const JUPITER_DCA_KEEPER3: DexProgram = DexProgram {
        id: "DCAK36VfExkPdAkYUQg6ewgxyinvcEyPLyHjRbmveKFw",
        name: "JupiterDcaKeeper3",
        tags: &["route"],
    };
    pub const JUPITER_LIMIT_ORDER: DexProgram = DexProgram {
        id: "jupoNjAxXgZ4rjzxzPMP4oxduvQsQtZzyknqvzYNrNu",
        name: "JupiterLimit",
        tags: &["route"],
    };
    pub const JUPITER_LIMIT_ORDER_V2: DexProgram = DexProgram {
        id: "j1o2qRpjcyUwEvwtcfhEQefh773ZgjxcVRry7LDqg5X",
        name: "JupiterLimitV2",
        tags: &["route"],
    };
    pub const JUPITER_VA: DexProgram = DexProgram {
        id: "VALaaymxQh2mNy2trH9jUqHT1mTow76wpTcGmSWSwJe",
        name: "JupiterVA",
        tags: &["route"],
    };
    pub const RAYDIUM_ROUTE: DexProgram = DexProgram {
        id: "routeUGWgWzqBWFcrCfv8tritsqukccJPu3q5GPP3xS",
        name: "RaydiumRoute",
        tags: &["route"],
    };
    pub const RAYDIUM_V4: DexProgram = DexProgram {
        id: "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",
        name: "RaydiumV4",
        tags: &["amm"],
    };
    pub const RAYDIUM_AMM: DexProgram = DexProgram {
        id: "5quBtoiQqxF9Jv6KYKctB59NT3gtJD2Y65kdnB1Uev3h",
        name: "RaydiumAMM",
        tags: &["amm"],
    };
    pub const RAYDIUM_CPMM: DexProgram = DexProgram {
        id: "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C",
        name: "RaydiumCPMM",
        tags: &["amm"],
    };
    pub const RAYDIUM_CL: DexProgram = DexProgram {
        id: "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK",
        name: "RaydiumCL",
        tags: &["amm"],
    };
    pub const RAYDIUM_LCP: DexProgram = DexProgram {
        id: "LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj",
        name: "RaydiumLaunchpad",
        tags: &["amm"],
    };
    pub const ORCA: DexProgram = DexProgram {
        id: "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc",
        name: "Orca",
        tags: &["amm"],
    };
    pub const METEORA: DexProgram = DexProgram {
        id: "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo",
        name: "MeteoraDLMM",
        tags: &["amm"],
    };
    pub const METEORA_DAMM: DexProgram = DexProgram {
        id: "Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB",
        name: "MeteoraDamm",
        tags: &["amm"],
    };
    pub const METEORA_DAMM_V2: DexProgram = DexProgram {
        id: "cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG",
        name: "MeteoraDammV2",
        tags: &["amm"],
    };
    pub const METEORA_DBC: DexProgram = DexProgram {
        id: "dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN",
        name: "MeteoraDBC",
        tags: &["amm"],
    };
    pub const PUMP_FUN: DexProgram = DexProgram {
        id: "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P",
        name: "Pumpfun",
        tags: &["amm"],
    };
    pub const PUMP_SWAP: DexProgram = DexProgram {
        id: "pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA",
        name: "Pumpswap",
        tags: &["amm"],
    };
    pub const MOONIT: DexProgram = DexProgram {
        id: "MoonCVVNZFSYkqNXP6bxHLPL6QQJiMagDL3qcqUQTrG",
        name: "Moonit",
        tags: &["amm"],
    };
    pub const BOOP_FUN: DexProgram = DexProgram {
        id: "boop8hVGQGqehUK2iVEMEnMrL5RbjywRzHKBmBE7ry4",
        name: "Boopfun",
        tags: &["amm"],
    };
    pub const SUGAR: DexProgram = DexProgram {
        id: "deus4Bvftd5QKcEkE5muQaWGWDoma8GrySvPFrBPjhS",
        name: "Sugar",
        tags: &["amm"],
    };
    pub const HEAVEN: DexProgram = DexProgram {
        id: "HEAVENoP2qxoeuF8Dj2oT1GHEnu49U5mJYkdeC8BAX2o",
        name: "Heaven",
        tags: &["amm"],
    };
}

pub fn get_program_name(program_id: &str) -> &'static str {
    let all: &[(&DexProgram, &str)] = &[
        (&dex_programs::JUPITER, dex_programs::JUPITER.name),
        (&dex_programs::JUPITER_DCA, dex_programs::JUPITER_DCA.name),
        (&dex_programs::JUPITER_VA, dex_programs::JUPITER_VA.name),
        (&dex_programs::JUPITER_LIMIT_ORDER_V2, dex_programs::JUPITER_LIMIT_ORDER_V2.name),
        (&dex_programs::RAYDIUM_ROUTE, dex_programs::RAYDIUM_ROUTE.name),
        (&dex_programs::RAYDIUM_V4, dex_programs::RAYDIUM_V4.name),
        (&dex_programs::RAYDIUM_AMM, dex_programs::RAYDIUM_AMM.name),
        (&dex_programs::RAYDIUM_CPMM, dex_programs::RAYDIUM_CPMM.name),
        (&dex_programs::RAYDIUM_CL, dex_programs::RAYDIUM_CL.name),
        (&dex_programs::RAYDIUM_LCP, dex_programs::RAYDIUM_LCP.name),
        (&dex_programs::ORCA, dex_programs::ORCA.name),
        (&dex_programs::METEORA, dex_programs::METEORA.name),
        (&dex_programs::METEORA_DAMM, dex_programs::METEORA_DAMM.name),
        (&dex_programs::METEORA_DAMM_V2, dex_programs::METEORA_DAMM_V2.name),
        (&dex_programs::METEORA_DBC, dex_programs::METEORA_DBC.name),
        (&dex_programs::PUMP_FUN, dex_programs::PUMP_FUN.name),
        (&dex_programs::PUMP_SWAP, dex_programs::PUMP_SWAP.name),
        (&dex_programs::MOONIT, dex_programs::MOONIT.name),
        (&dex_programs::BOOP_FUN, dex_programs::BOOP_FUN.name),
        (&dex_programs::SUGAR, dex_programs::SUGAR.name),
        (&dex_programs::HEAVEN, dex_programs::HEAVEN.name),
    ];
    for (prog, name) in all {
        if prog.id == program_id {
            return name;
        }
    }
    "Unknown"
}

pub const SYSTEM_PROGRAMS: &[&str] = &[
    "ComputeBudget111111111111111111111111111111",
    "11111111111111111111111111111111",
    TOKEN_PROGRAM_ID,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
    "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX",
];

pub const SKIP_PROGRAM_IDS: &[&str] = &[
    "pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ",
];

pub const FEE_ACCOUNTS: &[&str] = &[
    "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5",
    "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe",
    "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY",
    "ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49",
    "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh",
    "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt",
    "DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL",
    "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT",
    "45ruCyfdRkWpRNGEqWzjCiXRHkZs8WXCLQ67Pnpye7Hp",
    "39azUYFWPz3VHgKCf3VChUwbpURdCHRxjWVowf5jUJjg",
    "FWsW1xNtWscwNmKv6wVsU1iTzRN6wmmk3MjxRP5tT7hz",
    "G5UZAVbAf46s7cKWoyKu8kYTip9DGTpbLZ2qa9Aq69dP",
    "7hTckgnGnLQR6sdH7YkqFTAA7VwTfYFaZ6EhEsU3saCX",
    "9rPYyANsfQZw3DnDmKE3YCQF5E8oD89UXoHn9JFEhJUz",
    "7VtfL8fvgNfhz17qKRMjzQEXgbdpnHHHQRh54R9jP2RJ",
    "AVmoTthdrX6tKt4nDjco2D775W2YK3sDhxPcMmzUAmTY",
    "62qc2CNXwrYqQScmEdiZFFAnJR262PxWEuNQtxfafNgV",
    "JCRGumoE9Qi5BBgULTgdgTLjSgkCMSbF62ZZfGs84JeU",
    "CebN5WGQ4jvEPvsVU4EoHEpgzq1VV7AbicfhtW4xC9iM",
    "AVUCZyuT35YSuj4RH7fwiyPu82Djn2Hfg7y2ND2XcnZH",
    "BUX7s2ef2htTGb2KKoPHWkmzxPj4nTWMWRgs5CSbQxf9",
    "CdQTNULjDiTsvyR5UKjYBMqWvYpxXj6HY4m6atm2hErk",
];

// SPL Token instruction discriminators (first byte)
pub mod spl_token_instruction {
    pub const INITIALIZE_MINT: u8 = 0;
    pub const INITIALIZE_ACCOUNT: u8 = 1;
    pub const TRANSFER: u8 = 3;
    pub const MINT_TO: u8 = 7;
    pub const BURN: u8 = 8;
    pub const TRANSFER_CHECKED: u8 = 12;
    pub const MINT_TO_CHECKED: u8 = 14;
    pub const BURN_CHECKED: u8 = 15;
    pub const CLOSE_ACCOUNT: u8 = 9;
}

pub mod system_instruction {
    pub const TRANSFER: u8 = 2;
}

// Discriminators (first N bytes of instruction data)
pub mod discriminators {
    // Jupiter route event (16 bytes)
    pub const JUPITER_ROUTE_EVENT: [u8; 16] = [
        228, 69, 165, 46, 81, 203, 154, 29, 64, 198, 205, 232, 38, 8, 113, 226,
    ];
    // Pumpfun
    pub const PUMPFUN_CREATE: [u8; 8] = [24, 30, 200, 40, 5, 28, 7, 119];
    pub const PUMPFUN_BUY: [u8; 8] = [102, 6, 61, 18, 1, 218, 235, 234];
    pub const PUMPFUN_SELL: [u8; 8] = [51, 230, 133, 164, 1, 127, 131, 173];
    // Pumpswap
    pub const PUMPSWAP_BUY: [u8; 8] = [102, 6, 61, 18, 1, 218, 235, 234];
    pub const PUMPSWAP_SELL: [u8; 8] = [51, 230, 133, 164, 1, 127, 131, 173];
    // Raydium CPMM
    pub const RAYDIUM_CPMM_CREATE: [u8; 8] = [175, 175, 109, 31, 13, 152, 155, 237];
    pub const RAYDIUM_CPMM_ADD_LIQUIDITY: [u8; 8] = [242, 35, 198, 137, 82, 225, 242, 182];
    pub const RAYDIUM_CPMM_REMOVE_LIQUIDITY: [u8; 8] = [183, 18, 70, 156, 148, 109, 161, 34];
    // Meteora DLMM add liquidity
    pub const METEORA_DLMM_ADD_LIQUIDITY: [u8; 8] = [181, 157, 89, 67, 143, 182, 52, 72];
    pub const METEORA_DLMM_REMOVE_LIQUIDITY: [u8; 8] = [80, 85, 209, 72, 24, 206, 177, 108];
    // Orca
    pub const ORCA_CREATE: [u8; 8] = [242, 29, 134, 48, 58, 110, 14, 60];
    pub const ORCA_CREATE2: [u8; 8] = [212, 47, 95, 92, 114, 102, 131, 250];
    pub const ORCA_INCREASE_LIQUIDITY: [u8; 8] = [46, 156, 243, 118, 13, 205, 251, 178];
    pub const ORCA_INCREASE_LIQUIDITY2: [u8; 8] = [133, 29, 89, 223, 69, 238, 176, 10];
    pub const ORCA_DECREASE_LIQUIDITY: [u8; 8] = [160, 38, 208, 111, 104, 91, 44, 1];
    // Raydium (1 byte)
    pub const RAYDIUM_CREATE: [u8; 1] = [1];
    pub const RAYDIUM_ADD_LIQUIDITY: [u8; 1] = [3];
    pub const RAYDIUM_REMOVE_LIQUIDITY: [u8; 1] = [4];
    // Meteora DAMM
    pub const METEORA_DAMM_CREATE: [u8; 8] = [7, 166, 138, 171, 206, 171, 236, 244];
    pub const METEORA_DAMM_ADD: [u8; 8] = [168, 227, 50, 62, 189, 171, 84, 176];
    pub const METEORA_DAMM_REMOVE: [u8; 8] = [133, 109, 44, 179, 56, 238, 114, 33];
    // Meteora DAMM V2
    pub const METEORA_DAMM_V2_INIT: [u8; 8] = [95, 180, 10, 172, 84, 174, 232, 40];
    pub const METEORA_DAMM_V2_ADD: [u8; 8] = [181, 157, 89, 67, 143, 182, 52, 72];
    pub const METEORA_DAMM_V2_REMOVE: [u8; 8] = [80, 85, 209, 72, 24, 206, 177, 108];
    // Pumpfun / Pumpswap 16-byte event discriminators
    pub const PUMPFUN_TRADE_EVENT: [u8; 16] = [
        228, 69, 165, 46, 81, 203, 154, 29, 189, 219, 127, 211, 78, 230, 97, 238,
    ];
    pub const PUMPFUN_CREATE_EVENT: [u8; 16] = [
        228, 69, 165, 46, 81, 203, 154, 29, 27, 114, 169, 77, 222, 235, 99, 118,
    ];
    pub const PUMPSWAP_BUY_EVENT: [u8; 16] = [
        228, 69, 165, 46, 81, 203, 154, 29, 103, 244, 82, 31, 44, 245, 119, 119,
    ];
    pub const PUMPSWAP_SELL_EVENT: [u8; 16] = [
        228, 69, 165, 46, 81, 203, 154, 29, 62, 47, 55, 10, 165, 3, 220, 42,
    ];
}
