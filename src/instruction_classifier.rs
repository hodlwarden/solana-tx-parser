//! Classifies instructions by program ID (outer + inner).

use crate::constants::{SKIP_PROGRAM_IDS, SYSTEM_PROGRAMS};
use crate::transaction_adapter::TransactionAdapter;
use crate::types::{ClassifiedInstruction, ParsedInstruction};
use std::collections::HashMap;

pub struct InstructionClassifier<'a> {
    instruction_map: HashMap<String, Vec<ClassifiedInstruction>>,
    _adapter: &'a TransactionAdapter<'a>,
}

impl<'a> InstructionClassifier<'a> {
    pub fn new(adapter: &'a TransactionAdapter<'a>) -> Self {
        let mut classifier = Self {
            instruction_map: HashMap::new(),
            _adapter: adapter,
        };
        classifier.classify_instructions(adapter);
        classifier
    }

    fn classify_instructions(&mut self, adapter: &TransactionAdapter<'a>) {
        for (outer_index, raw) in adapter.raw_instructions().iter().enumerate() {
            let program_id = adapter.get_instruction_program_id(raw);
            self.add_instruction(ClassifiedInstruction {
                instruction: ParsedInstruction {
                    program_id: program_id.clone(),
                    accounts: raw
                        .account_key_indexes
                        .iter()
                        .filter_map(|&i| adapter.get_account_key(i as usize))
                        .collect(),
                    data: raw.data.clone(),
                    parsed: None,
                },
                program_id,
                outer_index,
                inner_index: None,
            });
        }
        if let Some(inner) = adapter.raw_inner_instructions() {
            for set in inner {
                for (inner_index, raw) in set.instructions.iter().enumerate() {
                    let program_id = adapter.get_instruction_program_id(raw);
                    self.add_instruction(ClassifiedInstruction {
                        instruction: ParsedInstruction {
                            program_id: program_id.clone(),
                            accounts: raw
                                .account_key_indexes
                                .iter()
                                .filter_map(|&i| adapter.get_account_key(i as usize))
                                .collect(),
                            data: raw.data.clone(),
                            parsed: None,
                        },
                        program_id,
                        outer_index: set.index as usize,
                        inner_index: Some(inner_index),
                    });
                }
            }
        }
    }

    fn add_instruction(&mut self, classified: ClassifiedInstruction) {
        if classified.program_id.is_empty() {
            return;
        }
        self.instruction_map
            .entry(classified.program_id.clone())
            .or_default()
            .push(classified);
    }

    pub fn get_instructions(&self, program_id: &str) -> Vec<ClassifiedInstruction> {
        self.instruction_map
            .get(program_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_all_program_ids(&self) -> Vec<String> {
        self.instruction_map
            .keys()
            .filter(|id| {
                !SYSTEM_PROGRAMS.contains(&id.as_str()) && !SKIP_PROGRAM_IDS.contains(&id.as_str())
            })
            .cloned()
            .collect()
    }
}
