use std::collections::HashMap;

use eyre::Context;
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::{
    commitment_config::{CommitmentConfig, CommitmentLevel}, instruction::CompiledInstruction, pubkey::Pubkey, signature::Signature
};
use solana_transaction_status::{
    option_serializer::OptionSerializer, UiCompiledInstruction, UiInnerInstructions, UiInstruction,
    UiTransactionEncoding,
};

pub struct TransactionHandler {
    pub client: RpcClient,
}

impl TransactionHandler {
    pub fn new(rpc_url: String) -> Self {
        Self { client: RpcClient::new(rpc_url) }
    }

    pub fn handle_tx(&self, signature: &Signature) -> eyre::Result<()> {
        let tx = self
            .client
            .get_transaction_with_config(
                signature,
                RpcTransactionConfig {
                    commitment: Some(CommitmentConfig { commitment: CommitmentLevel::Confirmed }),
                    encoding: Some(UiTransactionEncoding::Base58),
                    max_supported_transaction_version: Some(0),
                },
            )
            .context(format!("failed to get tx: {:?}", signature.to_string()))?;

        match (tx.transaction.transaction.decode(), tx.transaction.meta) {
            (Some(version_tx), Some(tx_meta)) => {
                let mut pretty = format!(
                    "
tx signautre:           {}
tx status:              {}
                ",
                    signature.to_string(),
                    tx_meta.err.map_or("Success".to_string(), |e| e.to_string())
                );
                let accounts = version_tx.message.static_account_keys();
                pretty.push_str(
                    format!(
                        "
fee:                    {}",
                        tx_meta.fee
                    )
                    .as_str(),
                );
                pretty.push_str(
                    format!(
                        "
compute units consumed: {}",
                        tx_meta.compute_units_consumed.map_or(0, |c| c)
                    )
                    .as_str(),
                );

                pretty.push_str(
                    parse_instructions(
                        version_tx.message.instructions(),
                        accounts,
                        tx_meta.inner_instructions,
                    )?
                    .as_str(),
                );

                print!("{}", pretty);
            }
            _ => {
                println!("invalid tx!")
            }
        }

        eyre::Ok(())
    }
}

fn parse_instructions(
    instructions: &[CompiledInstruction],
    accounts: &[Pubkey],
    inner_instructions: OptionSerializer<Vec<UiInnerInstructions>>,
) -> eyre::Result<String> {
    let mut inner_instruction_map = HashMap::<u8, Vec<UiInstruction>>::new();

    if let OptionSerializer::Some(inners) = &inner_instructions {
        for inner_ins in inners {
            inner_instruction_map.insert(inner_ins.index, inner_ins.instructions.clone());
        }
    }

    let mut pretty: String = "\nInstructions details:".to_string();
    for instruction_idx in 0..instructions.len() {
        let instruction_id = instruction_idx + 1;
        let instruction = &instructions[instruction_idx];
        if instruction.program_id_index as usize >= accounts.len() {
            continue;
        }
        let program_id = instruction.program_id(accounts);
        pretty.push_str(
            format!("\n#{} - interact with program id: {}", instruction_id, program_id.to_string())
                .as_str(),
        );

        let account_length = instruction.accounts.len();
        let account_str_length = account_length.to_string().len();

        if account_length > 0 {
            pretty.push_str("\ninput accounts:");
            for idx in 0..account_length {
                pretty.push_str(
                    format!("\n[{:0>account_str_length$}] - {}", idx, accounts[idx].to_string())
                        .as_str(),
                )
            }
        }

        pretty.push_str(format!("\ninstruction data: {}", bs58::encode(instruction.data.clone()).into_string()).as_str());

        if let Some(instructions) = inner_instruction_map.get(&(instruction_idx as u8)) {
            let mut stack = vec![];
            let mut sub_instruction_idx = 1;
            for instruction in instructions {
                if let UiInstruction::Compiled(compiled_ins) = instruction {
                    if stack.len() == 0 {
                        stack.push(compiled_ins);
                        continue;
                    }
                    // compare stack height, push instruction into the stack if stack height is greater than the one in the top of the stack
                    let top = stack.get(stack.len() - 1).unwrap();
                    match (top.stack_height, compiled_ins.stack_height) {
                        (Some(top_stack_height), Some(cur_stack_height)) => {
                            if cur_stack_height > top_stack_height {
                                stack.push(compiled_ins);
                                continue;
                            }

                            let mut inner_calls = vec![];
                            while stack.len() > 0 {
                                inner_calls.push(stack.pop());
                            }

                            if !inner_calls.is_empty() {
                                let pretty_inner_calls_res = pretty_inner_calls(inner_calls, accounts, instruction_id, sub_instruction_idx);
                                pretty.push_str(
                                    &pretty_inner_calls_res.as_str()
                                );
                                sub_instruction_idx = sub_instruction_idx + 1;
                            }

                            stack.push(compiled_ins);
                        }
                        (_, _) => {}
                    }
                }
            }

            let mut inner_calls = vec![];
            while stack.len() > 0 {
                inner_calls.push(stack.pop());
            }

            if !inner_calls.is_empty() {
                let pretty_inner_calls_res = pretty_inner_calls(inner_calls, accounts, instruction_id, sub_instruction_idx);
                pretty.push_str(
                    &pretty_inner_calls_res.as_str()
                );
                // sub_instruction_idx = sub_instruction_idx + 1;
            }
        }
    }

    eyre::Ok(pretty)
}


fn pretty_inner_calls(
    inner_calls: Vec<Option<&UiCompiledInstruction>>,
    accounts: &[Pubkey],
    instruction_id: usize,
    sub_idx: u8,
) -> String {
    let mut tab_count = 1;
    let mut pretty = "".to_string();
    for call in inner_calls.iter().rev() {
        if let Some(inner_ins) = call {
            pretty.push_str(
                with_tab_space_prefix_ln(tab_count, format!("#{}.{}", instruction_id, sub_idx))
                    .as_str(),
            );
            pretty.push_str(
                with_tab_space_prefix_ln(
                    tab_count,
                    format!("interact with: {}", accounts[inner_ins.program_id_index as usize]),
                )
                .as_str(),
            );
            if !inner_ins.accounts.is_empty() {
                pretty.push_str(
                    with_tab_space_prefix_ln(tab_count, format!("input accounts:")).as_str(),
                )
            }
            for sub_idx in 0..inner_ins.accounts.len() {
                pretty.push_str(
                    with_tab_space_prefix_ln(
                        tab_count,
                        format!("[{}] - {}", sub_idx, accounts[inner_ins.accounts[sub_idx] as usize]),
                    )
                    .as_str(),
                );
            }

            pretty.push_str(with_tab_space_prefix_ln(tab_count, format!("instruction data: {}", inner_ins.data)).as_str());

            tab_count = tab_count + 1;
        }
    }
    pretty
}

fn with_tab_space_prefix_ln(subnumber: usize, content: String) -> String {
    format!("\n{}{}", "     ".repeat(subnumber), content.as_str())
}
