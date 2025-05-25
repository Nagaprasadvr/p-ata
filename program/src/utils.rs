use pinocchio::{account_info::AccountInfo, instruction::Signer, pubkey::Pubkey, ProgramResult};

pub fn create_pda_account(
    payer: &AccountInfo,
    rent_exempt_lamports: u64,
    space: u64,
    owner: &Pubkey,
    new_pda_account: &AccountInfo,
    new_pda_signer_seeds: &[Signer],
) -> ProgramResult {
    if new_pda_account.lamports() > 0 {
        let req_lamports = space.max(1).saturating_sub(new_pda_account.lamports());
        if req_lamports > 0 {
            pinocchio_system::instructions::Transfer {
                from: payer,
                to: new_pda_account,
                lamports: req_lamports,
            }
            .invoke()?;
        }

        pinocchio_system::instructions::Allocate {
            account: new_pda_account,
            space,
        }
        .invoke_signed(new_pda_signer_seeds)?;

        pinocchio_system::instructions::Assign {
            account: new_pda_account,
            owner,
        }
        .invoke_signed(new_pda_signer_seeds)
    } else {
        pinocchio_system::instructions::CreateAccount {
            from: payer,
            to: new_pda_account,
            lamports: rent_exempt_lamports,
            space,
            owner,
        }
        .invoke_signed(new_pda_signer_seeds)
    }
}
