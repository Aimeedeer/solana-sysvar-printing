use crate::ProgramInstruction;
use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

/// # Accounts
///
/// - 0: payer - writable, signer
/// - 1: system_program - executable
/// - 2: clock - executable
/// - 3: epoch_schedule - executable
/// - 4: instructions - executable
/// - 5: rent - executable
/// - 6: slot_hashes - executable
/// - 7: slot_history - executable
/// - 8: stake_history - executable
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct SysvarInstruction {
    pub test_amount: u64,
}

impl SysvarInstruction {
    pub fn build_instruction(payer: &Pubkey, program_id: &Pubkey) -> Result<Instruction> {
        let instr = SysvarInstruction { test_amount: 1_000 };
        let instr = ProgramInstruction::Sysvar(instr);

        let accounts = vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(sysvar::clock::ID, false),
            AccountMeta::new_readonly(sysvar::epoch_schedule::ID, false),
            AccountMeta::new_readonly(sysvar::instructions::ID, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
            AccountMeta::new_readonly(sysvar::slot_hashes::ID, false),
            AccountMeta::new_readonly(sysvar::slot_history::ID, false),
            AccountMeta::new_readonly(sysvar::stake_history::ID, false),
        ];

        Ok(Instruction::new_with_borsh(*program_id, &instr, accounts))
    }
}
