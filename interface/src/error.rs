//! Error types

use pinocchio::program_error::ProgramError;

/// Errors that may be returned by the Associated Token program.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AssociatedTokenAccountError {
    // 0
    /// Associated token account owner does not match address derivation
    InvalidOwner,
}
impl From<AssociatedTokenAccountError> for ProgramError {
    fn from(e: AssociatedTokenAccountError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
