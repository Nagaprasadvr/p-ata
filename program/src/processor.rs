use ata_interface::{
    address::get_associated_token_address_and_bump_seed_internal,
    error::AssociatedTokenAccountError, instruction::CreateMode,
};
use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};

use pinocchio_token::state::{Mint, TokenAccount};

use crate::utils::create_pda_account;

#[inline(always)]
pub fn process_create_associated_token_account(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let create_mode = CreateMode::try_from(instruction_data[0])?;

    let [funder_info, ata_info, wallet_acc_info, mint_info, _system_program, spl_token_program, _rem @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let spl_token_program_id = spl_token_program.key();

    let (associated_token_address, bump_seed) = get_associated_token_address_and_bump_seed_internal(
        wallet_acc_info.key(),
        mint_info.key(),
        program_id,
        spl_token_program_id,
    );

    if associated_token_address.as_ref() != ata_info.key() {
        return Err(ProgramError::InvalidSeeds);
    }

    if create_mode == CreateMode::Idempotent
        // TODO: add support for extensions later when token-2022 is complete (becuase deser will error if we use token-2022 program accounts)
        && *spl_token_program.key() == pinocchio_token::ID
        && unsafe { ata_info.owner() } == spl_token_program_id
    {
        if let Ok(ata_state) = TokenAccount::from_account_info(ata_info) {
            if ata_state.owner() != wallet_acc_info.key() {
                return Err(AssociatedTokenAccountError::InvalidOwner.into());
            }

            if ata_state.mint() != mint_info.key() {
                return Err(ProgramError::InvalidAccountData);
            }

            return Ok(());
        }
    }

    if *unsafe { ata_info.owner() } != pinocchio_system::ID {
        return Err(ProgramError::IllegalOwner);
    }

    let rent_exempt_lamports = Rent::get()?.minimum_balance(TokenAccount::LEN);

    let bump_seed = [bump_seed];

    let ata_signer_seeds = [
        Seed::from(wallet_acc_info.key()),
        Seed::from(spl_token_program.key()),
        Seed::from(mint_info.key()),
        Seed::from(&bump_seed),
    ];

    let ata_signers = [Signer::from(&ata_signer_seeds[..])];

    //TODO: get account_len with extensions when token-2022 is done

    create_pda_account(
        funder_info,
        rent_exempt_lamports,
        pinocchio_token::state::TokenAccount::LEN as u64,
        spl_token_program_id,
        ata_info,
        ata_signers.as_ref(),
    )?;

    msg!("Initialize the associated token account");

    //TODO: invoke initialize immutable owner extension

    pinocchio_token::instructions::InitializeAccount3 {
        account: ata_info,
        mint: mint_info,
        owner: wallet_acc_info.key(),
    }
    .invoke()?;

    Ok(())
}

#[inline(always)]
pub fn process_recover_nested(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let [nested_ata_info, nested_mint_info, dest_ata_info, owner_ata_info, owner_mint_info, wallet_acc_info, spl_token_program, _rem @ ..] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    let spl_token_program_id = spl_token_program.key();

    let (owner_ata_address, bump_seed) = get_associated_token_address_and_bump_seed_internal(
        wallet_acc_info.key(),
        owner_mint_info.key(),
        program_id,
        spl_token_program.key(),
    );

    if owner_ata_address != *owner_ata_info.key() {
        return Err(ProgramError::InvalidSeeds);
    }

    // Check nested address derivation
    let (nested_ata_address, _) = get_associated_token_address_and_bump_seed_internal(
        owner_ata_info.key(),
        nested_mint_info.key(),
        program_id,
        spl_token_program_id,
    );
    if nested_ata_address != *nested_ata_info.key() {
        return Err(ProgramError::InvalidSeeds);
    }

    // Check destination address derivation
    let (destination_ata_address, _) = get_associated_token_address_and_bump_seed_internal(
        wallet_acc_info.key(),
        nested_mint_info.key(),
        program_id,
        spl_token_program_id,
    );
    if destination_ata_address != *dest_ata_info.key() {
        return Err(ProgramError::InvalidSeeds);
    }

    if !wallet_acc_info.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if unsafe { owner_mint_info.owner() } != spl_token_program_id {
        return Err(ProgramError::IllegalOwner);
    }

    let (amount, decimals) = {
        // Check owner associated token account data
        if unsafe { owner_ata_info.owner() } != spl_token_program_id {
            msg!("Owner associated token account not owned by provided token program, recreate the owner associated token account first");
            return Err(ProgramError::IllegalOwner);
        }

        let owner_ata_state = TokenAccount::from_account_info(owner_ata_info)?;
        if owner_ata_state.owner() != wallet_acc_info.key() {
            return Err(AssociatedTokenAccountError::InvalidOwner.into());
        }

        // Check nested associated token account data
        if unsafe { nested_ata_info.owner() } != spl_token_program_id {
            return Err(ProgramError::IllegalOwner);
        }

        let nested_ata_state = TokenAccount::from_account_info(nested_ata_info)?;
        if nested_ata_state.owner() != owner_ata_info.key() {
            return Err(AssociatedTokenAccountError::InvalidOwner.into());
        }
        let amount = nested_ata_state.amount();

        // Check nested token mint data
        if unsafe { nested_mint_info.owner() } != spl_token_program_id {
            return Err(ProgramError::IllegalOwner);
        }
        let nested_mint_state = Mint::from_account_info(nested_mint_info)?;
        let decimals = nested_mint_state.decimals();
        (amount, decimals)
    };

    let bump_seed = [bump_seed];

    let owner_signer_ata_seeds = [
        Seed::from(wallet_acc_info.key()),
        Seed::from(spl_token_program.key()),
        Seed::from(owner_mint_info.key()),
        Seed::from(&bump_seed),
    ];

    let owner_ata_signers = [Signer::from(&owner_signer_ata_seeds[..])];

    pinocchio_token::instructions::TransferChecked {
        from: nested_ata_info,
        to: dest_ata_info,
        mint: nested_mint_info,
        authority: owner_ata_info,
        amount,
        decimals,
    }
    .invoke_signed(owner_ata_signers.as_slice())?;

    pinocchio_token::instructions::CloseAccount {
        account: nested_ata_info,
        destination: wallet_acc_info,
        authority: owner_ata_info,
    }
    .invoke_signed(owner_ata_signers.as_slice())?;

    Ok(())
}
