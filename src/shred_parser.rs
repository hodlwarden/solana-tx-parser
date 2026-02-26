//! Parser for shred-stream / pre-execution instruction analysis.
//! Parses instructions by DEX without full transaction meta.

use crate::constants::get_program_name;
use crate::instruction_classifier::InstructionClassifier;
use crate::transaction_adapter::TransactionAdapter;
use crate::types::{ParseConfig, ParseShredResult};
use std::collections::HashMap;

pub struct ShredParser;

impl ShredParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse instructions from transaction (e.g. from shred stream) grouped by DEX name.
    pub fn parse_all(
        &self,
        tx: &crate::types::SolanaTransactionInput,
        config: Option<ParseConfig>,
    ) -> ParseShredResult {
        let config = config.unwrap_or_default();
        let mut result = ParseShredResult {
            state: true,
            signature: String::new(),
            instructions: HashMap::new(),
            msg: None,
        };
        let adapter = TransactionAdapter::new(tx, Some(config.clone()));
        result.signature = adapter.signature();
        let classifier = InstructionClassifier::new(&adapter);
        let all_program_ids = classifier.get_all_program_ids();
        if let Some(ref filter) = config.program_ids {
            if !all_program_ids.iter().any(|id| filter.contains(id)) {
                return result;
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
            let name = get_program_name(program_id);
            result.instructions.insert(
                name.to_string(),
                instructions
                    .into_iter()
                    .map(|ci| serde_json::json!({ "programId": ci.program_id, "outerIndex": ci.outer_index, "innerIndex": ci.inner_index }))
                    .collect(),
            );
        }
        result
    }
}

impl Default for ShredParser {
    fn default() -> Self {
        Self::new()
    }
}
