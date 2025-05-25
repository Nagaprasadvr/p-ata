use pinocchio::{
    account_info::AccountInfo, default_panic_handler, no_allocator, program_entrypoint,
    program_error::ProgramError, pubkey::Pubkey, ProgramResult,
};

use crate::processor::*;

program_entrypoint!(process_instruction);
// Do not allocate memory.
no_allocator!();
// Use the default panic handler.
default_panic_handler!();

/// Process an instruction.
///
/// Instructions
///
/// - `0`: `Create`
/// - `1`: `CreateIdempotent`
/// - `2`: `RecoverNested`
#[inline(always)]
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (discriminator, instruction_data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match *discriminator {
        // 0 - Create
        0 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: Create");

            process_create_associated_token_account(program_id, accounts, instruction_data)
        }

        // 3 - CreateIdempotent
        1 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: CreateIdempotent");

            process_create_associated_token_account(program_id, accounts, instruction_data)
        }
        // 7 - RecoverNested
        2 => {
            #[cfg(feature = "logging")]
            pinocchio::msg!("Instruction: RecovereNested");

            process_recover_nested(program_id, accounts)
        }
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
