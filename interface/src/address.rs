//! Address derivation functions

use pinocchio::pubkey::Pubkey;

/// Derives the associated token account address and bump seed
/// for the given wallet address, token mint and token program id
#[inline(always)]
pub fn get_associated_token_address_and_bump_seed(
    wallet_address: &Pubkey,
    token_mint_address: &Pubkey,
    program_id: &Pubkey,
    token_program_id: &Pubkey,
) -> (Pubkey, u8) {
    get_associated_token_address_and_bump_seed_internal(
        wallet_address,
        token_mint_address,
        program_id,
        token_program_id,
    )
}

mod inline_spl_token {
    pinocchio_pubkey::declare_id!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
}

/// Derives the associated token account address for the given wallet address
/// and token mint
#[inline(always)]
pub fn get_associated_token_address(
    wallet_address: &Pubkey,
    token_mint_address: &Pubkey,
) -> Pubkey {
    get_associated_token_address_with_program_id(
        wallet_address,
        token_mint_address,
        &inline_spl_token::ID,
    )
}

/// Derives the associated token account address for the given wallet address,
/// token mint and token program id
#[inline(always)]
pub fn get_associated_token_address_with_program_id(
    wallet_address: &Pubkey,
    token_mint_address: &Pubkey,
    token_program_id: &Pubkey,
) -> Pubkey {
    get_associated_token_address_and_bump_seed(
        wallet_address,
        token_mint_address,
        &crate::program::id(),
        token_program_id,
    )
    .0
}

/// For internal use only.
#[inline(always)]
#[doc(hidden)]
pub fn get_associated_token_address_and_bump_seed_internal(
    wallet_address: &Pubkey,
    token_mint_address: &Pubkey,
    program_id: &Pubkey,
    token_program_id: &Pubkey,
) -> (Pubkey, u8) {
    pinocchio::pubkey::find_program_address(
        &[wallet_address, token_program_id, token_mint_address],
        program_id,
    )
}
